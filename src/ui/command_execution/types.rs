//! Types and enums for command execution.

/// Command execution context (privilege, helpers, etc.)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandType {
    /// Normal command execution
    Normal,
    /// Privileged command execution (via pkexec)
    Privileged,
    /// AUR helper command execution (paru/yay)
    Aur,
}

/// Command to execute with metadata
#[derive(Clone, Debug)]
pub struct CommandStep {
    pub command_type: CommandType,
    pub command: String,
    pub args: Vec<String>,
    pub friendly_name: String,
}

/// Result of command execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command failed with optional exit code
    Failure { exit_code: Option<i32> },
}

impl CommandStep {
    /// Create a new command with an explicit command type
    pub fn new(
        command_type: CommandType,
        command: &str,
        args: &[&str],
        friendly_name: &str,
    ) -> Self {
        Self {
            command_type,
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            friendly_name: friendly_name.to_string(),
        }
    }

    /// Convenience helper for normal commands
    pub fn normal(command: &str, args: &[&str], friendly_name: &str) -> Self {
        Self::new(CommandType::Normal, command, args, friendly_name)
    }

    /// Convenience helper for privileged commands (runs through pkexec)
    pub fn privileged(command: &str, args: &[&str], friendly_name: &str) -> Self {
        Self::new(CommandType::Privileged, command, args, friendly_name)
    }

    /// Convenience helper for AUR helper commands (paru/yay)
    pub fn aur(args: &[&str], friendly_name: &str) -> Self {
        Self::new(CommandType::Aur, "aur", args, friendly_name)
    }
}

impl CommandResult {
    /// Check if the result indicates success
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, CommandResult::Success)
    }

    /// Check if the result indicates failure
    #[allow(dead_code)]
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }

    /// Get the exit code if this is a failure
    #[allow(dead_code)]
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandResult::Failure { exit_code } => *exit_code,
            _ => None,
        }
    }
}
