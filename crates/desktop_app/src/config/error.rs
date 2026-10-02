use std::{fmt, io, path::PathBuf, process::ExitStatus};

#[derive(Debug)]
pub enum ConfigIoError {
    ConfigPathUnavailable,
    InvalidConfigPath(PathBuf),
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    ReadConfig {
        path: PathBuf,
        source: io::Error,
    },
    WriteConfig {
        path: PathBuf,
        source: io::Error,
    },
    CreateTempFile {
        path: PathBuf,
        source: io::Error,
    },
    PersistTempFile {
        path: PathBuf,
        source: io::Error,
    },
    LaunchOpenCommand {
        command: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    OpenCommandFailed {
        command: &'static str,
        path: PathBuf,
        status: ExitStatus,
    },
}

impl fmt::Display for ConfigIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigPathUnavailable => f.write_str(t!("Unable to determine config file path")),
            Self::InvalidConfigPath(path) => f.write_str(&t!(
                "Invalid config file path: {path}",
                path = path.display()
            )),
            Self::CreateDir { path, source } => f.write_str(&t!(
                "Failed to create config directory '{path}': {source}",
                path = path.display(),
                source = source
            )),
            Self::ReadConfig { path, source } => f.write_str(&t!(
                "Failed to read config file '{path}': {source}",
                path = path.display(),
                source = source
            )),
            Self::WriteConfig { path, source } => f.write_str(&t!(
                "Failed to write config file '{path}': {source}",
                path = path.display(),
                source = source
            )),
            Self::CreateTempFile { path, source } => f.write_str(&t!(
                "Failed to create temp config file near '{path}': {source}",
                path = path.display(),
                source = source
            )),
            Self::PersistTempFile { path, source } => f.write_str(&t!(
                "Failed to persist config file '{path}': {source}",
                path = path.display(),
                source = source
            )),
            Self::LaunchOpenCommand {
                command,
                path,
                source,
            } => f.write_str(&t!(
                "Failed to launch '{command}' for '{path}': {source}",
                command = command,
                path = path.display(),
                source = source
            )),
            Self::OpenCommandFailed {
                command,
                path,
                status,
            } => f.write_str(&t!(
                "'{command}' failed for '{path}' with status {status}",
                command = command,
                path = path.display(),
                status = status
            )),
        }
    }
}

impl std::error::Error for ConfigIoError {}
