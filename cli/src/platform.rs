use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Unsupported operating system: {0}")]
    UnsupportedOS(String),
    #[error("Unsupported architecture: {0}")]
    UnsupportedArch(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OS {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arch {
    X86_64,
    Aarch64,
}

#[derive(Debug, Clone)]
pub struct Platform {
    pub os: OS,
    pub arch: Arch,
}

impl Platform {
    pub fn detect() -> Result<Self, PlatformError> {
        let os = match env::consts::OS {
            "linux" => OS::Linux,
            "macos" => OS::Macos,
            "windows" => OS::Windows,
            other => return Err(PlatformError::UnsupportedOS(other.to_string())),
        };

        let arch = match env::consts::ARCH {
            "x86_64" => Arch::X86_64,
            "aarch64" | "arm64" => Arch::Aarch64,
            other => return Err(PlatformError::UnsupportedArch(other.to_string())),
        };

        Ok(Platform { os, arch })
    }

    pub fn target_triple(&self) -> &'static str {
        match (&self.os, &self.arch) {
            (OS::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
            (OS::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu",
            (OS::Macos, Arch::X86_64) => "x86_64-apple-darwin",
            (OS::Macos, Arch::Aarch64) => "aarch64-apple-darwin",
            (OS::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
            (OS::Windows, Arch::Aarch64) => "aarch64-pc-windows-msvc",
        }
    }

    pub fn archive_extension(&self) -> &'static str {
        match self.os {
            OS::Windows => ".zip",
            _ => ".tar.gz",
        }
    }

    pub fn binary_extension(&self) -> &'static str {
        match self.os {
            OS::Windows => ".exe",
            _ => "",
        }
    }

    /// Returns the name of the release archive that bundles every Saorsa binary
    /// for this platform.
    pub fn archive_name(&self) -> String {
        format!(
            "saorsa-cli-{}{}",
            self.target_triple(),
            self.archive_extension()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_archive_names() {
        let platform = Platform {
            os: OS::Macos,
            arch: Arch::Aarch64,
        };
        assert_eq!(
            platform.archive_name(),
            "saorsa-cli-aarch64-apple-darwin.tar.gz"
        );

        let platform = Platform {
            os: OS::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(
            platform.archive_name(),
            "saorsa-cli-x86_64-pc-windows-msvc.zip"
        );
    }
}
