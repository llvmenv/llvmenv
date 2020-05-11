use std::{io, path::*, process};
use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error while accessing {path}: {source:?}")]
    FileIo { path: PathBuf, source: io::Error },

    #[error(transparent)]
    FileIoExtra {
        #[from]
        source: fs_extra::error::Error,
    },

    #[error("Unsupported OS which cannot get (config|cache|data) directory")]
    UnsupportedOS,

    #[error("Unsupported cmake generator: {generator}")]
    UnsupportedGenerator { generator: String },

    #[error("Configure file already exists: {path}")]
    ConfigureAlreadyExists { path: PathBuf },

    #[error("Failed to get LLVM version: {version}")]
    InvalidVersion { version: String },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error(transparent)]
    InvalidTOML {
        #[from]
        source: toml::de::Error,
    },

    #[error("Entry {name} is invalid: {message}")]
    InvalidEntry { name: String, message: String },

    #[error("HTTP request does not succeed with {status}: {url}")]
    HttpError {
        url: String,
        status: reqwest::StatusCode,
    },

    #[error(transparent)]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },

    #[error("External command exit with error-code({errno}): {cmd}")]
    CommandError {
        errno: i32,
        cmd: String,
        stdout: Option<String>,
        stderr: Option<String>,
    },

    #[error("External command not found: {cmd}")]
    CommandNotFound { cmd: String },

    #[error("External command has been terminated by signal: {cmd}")]
    CommandTerminatedBySignal {
        cmd: String,
        stdout: Option<String>,
        stderr: Option<String>,
    },
}

impl Error {
    pub fn invalid_version(version: &str) -> Self {
        Error::InvalidVersion {
            version: version.into(),
        }
    }
}

pub trait FileIoConvert<T> {
    fn with(self, path: impl AsRef<Path>) -> Result<T>;
}

impl<T> FileIoConvert<T> for ::std::result::Result<T, io::Error> {
    fn with(self, path: impl AsRef<Path>) -> Result<T> {
        self.map_err(|source| Error::FileIo {
            source,
            path: path.as_ref().into(),
        })
    }
}

pub trait CommandExt {
    fn silent(&mut self) -> &mut Self;
    fn check_run(&mut self) -> Result<()>;
    fn check_output(&mut self) -> Result<(String, String)>;
}

impl CommandExt for process::Command {
    fn silent(&mut self) -> &mut Self {
        self.stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
    }

    fn check_run(&mut self) -> Result<()> {
        let cmd = format!("{:?}", self);
        let st = self
            .status()
            .map_err(|_| Error::CommandNotFound { cmd: cmd.clone() })?;
        match st.code() {
            Some(errno) => {
                if errno != 0 {
                    Err(Error::CommandError {
                        errno,
                        cmd,
                        stdout: None,
                        stderr: None,
                    })
                } else {
                    Ok(())
                }
            }
            None => Err(Error::CommandTerminatedBySignal {
                cmd,
                stdout: None,
                stderr: None,
            }),
        }
    }

    fn check_output(&mut self) -> Result<(String, String)> {
        let cmd = format!("{:?}", self);
        let output = self
            .output()
            .map_err(|_| Error::CommandNotFound { cmd: cmd.clone() })?;
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
        match output.status.code() {
            Some(errno) => {
                if errno != 0 {
                    Err(Error::CommandError {
                        errno,
                        cmd,
                        stdout: Some(stdout),
                        stderr: Some(stderr),
                    })
                } else {
                    Ok((stdout, stderr))
                }
            }
            None => Err(Error::CommandTerminatedBySignal {
                cmd,
                stdout: Some(stdout),
                stderr: Some(stderr),
            }),
        }
    }
}
