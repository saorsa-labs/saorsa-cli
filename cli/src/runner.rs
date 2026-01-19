#[cfg(not(unix))]
use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
    /// Reserved for future non-zero exit code handling.
    #[allow(dead_code)]
    #[error("Failed to execute binary: {0}")]
    NonZeroExit(i32),
    #[error("Failed to exec binary: {0}")]
    ExecFailed(#[source] std::io::Error),
}

pub struct BinaryRunner;

impl BinaryRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run_interactive(&self, binary_path: &Path, args: Vec<String>) -> Result<()> {
        if !binary_path.exists() {
            return Err(RunnerError::BinaryNotFound(binary_path.display().to_string()).into());
        }

        tracing::info!("Running interactive binary: {:?}", binary_path);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let err = Command::new(binary_path).args(args).exec();
            return Err(RunnerError::ExecFailed(err).into());
        }

        #[cfg(not(unix))]
        {
            let status = Command::new(binary_path)
                .args(args)
                .status()
                .with_context(|| format!("Failed to execute binary: {}", binary_path.display()))?;

            if !status.success() {
                // The process has exited with a non-zero status code.
                if let Some(code) = status.code() {
                    if code != 0 && code != 130 {
                        // 130 is SIGINT (Ctrl+C)
                        tracing::warn!("Binary exited with code: {}", code);
                    }
                }
            }

            return Ok(());
        }

        #[allow(unreachable_code)]
        Ok(())
    }

    pub fn check_binary_exists(&self, binary_path: &Path) -> bool {
        binary_path.exists()
    }

    pub fn which(&self, binary_name: &str) -> Option<PathBuf> {
        use std::env;

        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(binary_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
        })
    }
}
