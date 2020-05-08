use std::process;
use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, anyhow::Error>;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Exit with error-code({errno}): {cmd}\n[stdout]\n{stdout}\n[stderr]\n{stderr}")]
    ErrorCode {
        errno: i32,
        cmd: String,
        stdout: String,
        stderr: String,
    },
    #[error("External command not found: {cmd}")]
    CommandNotFound { cmd: String },
    #[error("Terminated by signal: {cmd}\n[stdout]\n{stdout}\n[stderr]\n{stderr}")]
    TerminatedBySignal {
        cmd: String,
        stdout: String,
        stderr: String,
    },
}

pub trait CommandExt {
    fn silent(&mut self) -> &mut Self;
    fn check_run(&mut self) -> Result<()>;
}

impl CommandExt for process::Command {
    fn silent(&mut self) -> &mut Self {
        self.stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
    }
    fn check_run(&mut self) -> Result<()> {
        let cmd = format!("{:?}", self);
        let output = self
            .output()
            .map_err(|_| CommandError::CommandNotFound { cmd: cmd.clone() })?;
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
        match output.status.code() {
            Some(errno) => {
                if errno != 0 {
                    Err(CommandError::ErrorCode {
                        errno,
                        cmd,
                        stdout,
                        stderr,
                    }
                    .into())
                } else {
                    Ok(())
                }
            }
            None => Err(CommandError::TerminatedBySignal {
                cmd,
                stdout,
                stderr,
            }
            .into()),
        }
    }
}
