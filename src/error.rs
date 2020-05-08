use anyhow::Error;
use std::process;
use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;
pub type CommandResult = ::std::result::Result<(), CommandError>;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Exit with error-code({errno}): {cmd}")]
    ErrorCode { errno: i32, cmd: String },
    #[error("External command not found: {cmd}")]
    CommandNotFound { cmd: String },
    #[error("Terminated by signal: {cmd}")]
    TerminatedBySignal { cmd: String },
}

pub trait CommandExt {
    fn silent(&mut self) -> &mut Self;
    fn check_run(&mut self) -> CommandResult;
}

impl CommandExt for process::Command {
    fn silent(&mut self) -> &mut Self {
        self.stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
    }
    fn check_run(&mut self) -> CommandResult {
        let cmd = format!("{:?}", self);
        let st = self
            .status()
            .map_err(|_| CommandError::CommandNotFound { cmd: cmd.clone() })?;
        match st.code() {
            Some(errno) => {
                if errno != 0 {
                    Err(CommandError::ErrorCode { errno, cmd })
                } else {
                    Ok(())
                }
            }
            None => Err(CommandError::TerminatedBySignal { cmd }),
        }
    }
}
