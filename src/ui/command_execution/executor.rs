//! Command execution logic and stream handling.

use super::context::RunningCommandContext;
use super::types::{CommandResult, CommandStep, CommandType};
use super::widgets::CommandExecutionWidgets;
use crate::{aur_helper, utils};
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use log::info;
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

/// Execute a sequence of commands
pub fn execute_commands_sequence(
    widgets: Rc<CommandExecutionWidgets>,
    commands: Rc<Vec<CommandStep>>,
    index: usize,
    cancelled: Rc<RefCell<bool>>,
    on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
    current_process: Rc<RefCell<Option<gio::Subprocess>>>,
) {
    if *cancelled.borrow() {
        finalize_execution(&widgets, false, "Operation cancelled");
        if let Some(callback) = on_complete {
            callback(false);
        }
        return;
    }

    if index >= commands.len() {
        finalize_execution(&widgets, true, "All operations completed successfully!");
        if let Some(callback) = on_complete {
            callback(true);
        }
        return;
    }

    let cmd = &commands[index];
    let total = commands.len();

    widgets.update_progress(index + 1, total);
    widgets.set_title(&cmd.friendly_name);

    append_output(
        &widgets,
        &format!(
            "\n=== Step {}/{}: {} ===\n",
            index + 1,
            total,
            cmd.friendly_name
        ),
        false,
    );

    let (full_command, full_args) = match resolve_command(cmd) {
        Ok(result) => result,
        Err(err) => {
            append_output(&widgets, &format!("✗ {}\n", err), true);
            finalize_execution(&widgets, false, "Failed to prepare command");
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    info!("Executing: {} {:?}", full_command, full_args);

    let mut argv: Vec<OsString> = Vec::with_capacity(1 + full_args.len());
    argv.push(OsString::from(full_command.clone()));
    for arg in &full_args {
        argv.push(OsString::from(arg));
    }
    let argv_refs: Vec<&std::ffi::OsStr> = argv.iter().map(|s| s.as_os_str()).collect();

    let flags = gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_PIPE;
    let subprocess = match gio::Subprocess::newv(&argv_refs, flags) {
        Ok(proc) => proc,
        Err(err) => {
            append_output(
                &widgets,
                &format!("✗ Failed to start command: {}\n", err),
                true,
            );
            finalize_execution(&widgets, false, "Failed to start operation");
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    *current_process.borrow_mut() = Some(subprocess.clone());

    let context = RunningCommandContext::new(
        widgets.clone(),
        commands.clone(),
        index,
        cancelled.clone(),
        on_complete.clone(),
        current_process.clone(),
    );

    attach_stream_reader(&subprocess, context.clone(), false);
    attach_stream_reader(&subprocess, context.clone(), true);

    let wait_context = context.clone();
    let wait_subprocess = subprocess.clone();
    wait_subprocess
        .clone()
        .wait_async(None::<&gio::Cancellable>, move |result| match result {
            Ok(_) => {
                if wait_subprocess.is_successful() {
                    wait_context.set_exit_result(CommandResult::Success);
                } else {
                    wait_context.set_exit_result(CommandResult::Failure {
                        exit_code: Some(wait_subprocess.exit_status()),
                    });
                }
            }
            Err(err) => {
                append_output(
                    &wait_context.widgets,
                    &format!("✗ Failed to wait for command: {}\n", err),
                    true,
                );
                wait_context.set_exit_result(CommandResult::Failure { exit_code: None });
            }
        });
}

/// Resolve command with proper privilege escalation and helpers
pub fn resolve_command(command: &CommandStep) -> Result<(String, Vec<String>), String> {
    match command.command_type {
        CommandType::Normal => Ok((command.command.clone(), command.args.clone())),
        CommandType::Privileged => {
            let mut args = Vec::with_capacity(command.args.len() + 1);
            args.push(command.command.clone());
            args.extend(command.args.clone());
            Ok(("pkexec".to_string(), args))
        }
        CommandType::Aur => {
            let helper = aur_helper()
                .map(|h| h.to_string())
                .or_else(|| utils::detect_aur_helper().map(|h| h.to_string()))
                .ok_or_else(|| "AUR helper not initialized (paru or yay required).".to_string())?;
            let mut args = Vec::with_capacity(command.args.len() + 2);
            args.push("--sudo".to_string());
            args.push("pkexec".to_string());
            args.extend(command.args.clone());
            Ok((helper, args))
        }
    }
}

/// Attach stream reader for stdout or stderr
pub fn attach_stream_reader(
    subprocess: &gio::Subprocess,
    context: Rc<RunningCommandContext>,
    is_error_stream: bool,
) {
    let stream = if is_error_stream {
        subprocess.stderr_pipe()
    } else {
        subprocess.stdout_pipe()
    };

    if let Some(stream) = stream {
        let data_stream = gio::DataInputStream::new(&stream);
        read_stream(data_stream, context, is_error_stream);
    } else {
        context.mark_stream_done(is_error_stream);
    }
}

/// Read stream line by line asynchronously
fn read_stream(
    data_stream: gio::DataInputStream,
    context: Rc<RunningCommandContext>,
    is_error_stream: bool,
) {
    let stream_clone = data_stream.clone();
    data_stream.clone().read_line_utf8_async(
        glib::Priority::default(),
        None::<&gio::Cancellable>,
        move |res| match res {
            Ok(Some(line)) => {
                let mut text = line.to_string();
                text.push('\n');
                append_output(&context.widgets, &text, is_error_stream);
                read_stream(stream_clone.clone(), context.clone(), is_error_stream);
            }
            Ok(None) => {
                context.mark_stream_done(is_error_stream);
            }
            Err(err) => {
                append_output(
                    &context.widgets,
                    &format!("✗ Failed to read command output: {}\n", err),
                    true,
                );
                context.mark_stream_done(is_error_stream);
            }
        },
    );
}

/// Append output to the text buffer
pub fn append_output(widgets: &CommandExecutionWidgets, text: &str, is_error: bool) {
    let buffer = &widgets.output_buffer;
    let mut end_iter = buffer.end_iter();

    if is_error {
        if let Some(error_tag) = buffer.tag_table().lookup("error") {
            buffer.insert_with_tags(&mut end_iter, text, &[&error_tag]);
        } else {
            buffer.insert(&mut end_iter, text);
        }
    } else {
        buffer.insert(&mut end_iter, text);
    }

    // Auto-scroll to bottom
    let mark = buffer.create_mark(None, &buffer.end_iter(), false);
    widgets
        .output_view
        .scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
}

/// Finalize dialog with success or failure message
pub fn finalize_execution(widgets: &CommandExecutionWidgets, success: bool, message: &str) {
    use std::sync::atomic::Ordering;
    super::ACTION_RUNNING.store(false, Ordering::SeqCst);

    widgets.show_completion(success, message);

    if success {
        append_output(widgets, &format!("\n✓ {}\n", message), false);
    } else {
        append_output(widgets, &format!("\n✗ {}\n", message), true);
    }
}
