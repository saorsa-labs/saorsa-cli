use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::AsyncWriteExt;

use crate::platform::Platform;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("No matching asset found for platform")]
    NoMatchingAsset,
    #[error("No releases found")]
    NoReleases,
    #[error("Checksum verification failed: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Checksum not found for asset: {0}")]
    ChecksumNotFound(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub assets: Vec<GitHubAsset>,
    pub published_at: String,
    /// Release body/notes (may contain checksums for older releases)
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct Downloader {
    client: Client,
    repo_owner: String,
    repo_name: String,
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new(repo_owner: String, repo_name: String) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to find cache directory")?
            .join("saorsa-cli")
            .join("binaries");

        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        let client = Client::builder()
            .user_agent("saorsa-cli/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            repo_owner,
            repo_name,
            cache_dir,
        })
    }

    pub async fn get_latest_release(&self) -> Result<GitHubRelease, DownloadError> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            // Try to get all releases if latest doesn't exist
            let url = format!(
                "https://api.github.com/repos/{}/{}/releases",
                self.repo_owner, self.repo_name
            );

            let releases: Vec<GitHubRelease> = self.client.get(&url).send().await?.json().await?;

            releases.into_iter().next().ok_or(DownloadError::NoReleases)
        } else {
            Ok(response.json().await?)
        }
    }

    pub fn binary_path(&self, binary_name: &str, platform: &Platform) -> PathBuf {
        self.cache_dir
            .join(format!("{}{}", binary_name, platform.binary_extension()))
    }

    pub async fn download_binary(
        &self,
        binary_name: &str,
        platform: &Platform,
        force: bool,
    ) -> Result<PathBuf> {
        let binary_path = self.binary_path(binary_name, platform);

        if binary_path.exists() && !force {
            tracing::info!("Binary already exists at {:?}", binary_path);
            return Ok(binary_path);
        }

        let release = self
            .get_latest_release()
            .await
            .context("Failed to get latest release")?;

        let asset_name = platform.asset_name(binary_name);
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or(DownloadError::NoMatchingAsset)?;

        tracing::info!(
            "Downloading {} from {}",
            asset.name,
            asset.browser_download_url
        );

        let archive_path = self
            .download_asset(asset)
            .await
            .context("Failed to download asset")?;

        // Verify checksum if available
        match self.fetch_checksums(&release).await {
            Ok(checksums) => {
                if let Some(expected) = checksums.get(&asset.name) {
                    Self::verify_checksum(&archive_path, expected)
                        .context("Checksum verification failed")?;
                    tracing::info!("Checksum verified for {}", asset.name);
                } else {
                    tracing::warn!(
                        "No checksum found for {}, skipping verification",
                        asset.name
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Could not fetch checksums: {}, skipping verification", e);
            }
        }

        self.extract_binary(&archive_path, binary_name, platform)
            .await
            .context("Failed to extract binary")?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    async fn download_asset(&self, asset: &GitHubAsset) -> Result<PathBuf> {
        let archive_path = self.cache_dir.join(&asset.name);

        let response = self
            .client
            .get(&asset.browser_download_url)
            .send()
            .await
            .context("Failed to start download")?;

        let total_size = response.content_length().unwrap_or(asset.size);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );

        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .context("Failed to create archive file")?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to download chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");

        Ok(archive_path)
    }

    async fn extract_binary(
        &self,
        archive_path: &Path,
        binary_name: &str,
        platform: &Platform,
    ) -> Result<()> {
        let binary_path = self.binary_path(binary_name, platform);

        match platform.archive_extension() {
            ".tar.gz" => {
                use flate2::read::GzDecoder;
                use tar::Archive;

                let file = File::open(archive_path).context("Failed to open archive")?;
                let gz = GzDecoder::new(file);
                let mut archive = Archive::new(gz);

                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path = entry.path()?;

                    if let Some(name) = path.file_name() {
                        if name == binary_name {
                            let mut output = File::create(&binary_path)
                                .context("Failed to create binary file")?;
                            io::copy(&mut entry, &mut output)
                                .context("Failed to extract binary")?;
                            return Ok(());
                        }
                    }
                }

                anyhow::bail!("Binary {} not found in archive", binary_name);
            }
            ".zip" => {
                use zip::ZipArchive;

                let file = File::open(archive_path).context("Failed to open archive")?;
                let mut archive = ZipArchive::new(file)?;

                let binary_name_with_ext =
                    format!("{}{}", binary_name, platform.binary_extension());

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    if let Some(name) = Path::new(file.name()).file_name() {
                        if name == binary_name_with_ext.as_str() || name == binary_name {
                            let mut output = File::create(&binary_path)
                                .context("Failed to create binary file")?;
                            io::copy(&mut file, &mut output).context("Failed to extract binary")?;
                            return Ok(());
                        }
                    }
                }

                anyhow::bail!("Binary {} not found in archive", binary_name);
            }
            _ => anyhow::bail!("Unsupported archive format"),
        }
    }

    /// Fetch the CHECKSUMS.txt asset content from a release.
    async fn fetch_checksums(
        &self,
        release: &GitHubRelease,
    ) -> Result<HashMap<String, String>, DownloadError> {
        // Look for CHECKSUMS.txt asset
        let checksums_asset = release
            .assets
            .iter()
            .find(|a| a.name == "CHECKSUMS.txt")
            .ok_or_else(|| DownloadError::ChecksumNotFound("CHECKSUMS.txt".to_string()))?;

        // Download the checksums file
        let content = self
            .client
            .get(&checksums_asset.browser_download_url)
            .send()
            .await?
            .text()
            .await?;

        Ok(parse_checksums(&content))
    }

    /// Verify a file's SHA256 checksum.
    fn verify_checksum(path: &Path, expected: &str) -> Result<(), DownloadError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let actual = hex::encode(hasher.finalize());

        if actual != expected {
            return Err(DownloadError::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }
        Ok(())
    }
}

/// Parse sha256sum format: "hash  filename" (two spaces between hash and filename).
fn parse_checksums(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // sha256sum format: hash followed by two spaces then filename
            // Also handle single space for compatibility
            let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace()).collect();
            if parts.len() >= 2 {
                let hash = parts[0].trim();
                let filename = parts[1].trim();
                if !hash.is_empty() && !filename.is_empty() {
                    return Some((filename.to_string(), hash.to_string()));
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_checksums_standard_format() {
        // Standard sha256sum format with two spaces
        let content = "abc123def456789012345678901234567890123456789012345678901234  file1.tar.gz\n\
                       fedcba9876543210fedcba9876543210fedcba9876543210fedcba987654  file2.tar.gz\n";
        let checksums = parse_checksums(content);

        assert_eq!(checksums.len(), 2);
        assert_eq!(
            checksums.get("file1.tar.gz"),
            Some(&"abc123def456789012345678901234567890123456789012345678901234".to_string())
        );
        assert_eq!(
            checksums.get("file2.tar.gz"),
            Some(&"fedcba9876543210fedcba9876543210fedcba9876543210fedcba987654".to_string())
        );
    }

    #[test]
    fn test_parse_checksums_single_space() {
        // Some tools use single space
        let content = "abc123 file.tar.gz\n";
        let checksums = parse_checksums(content);

        assert_eq!(checksums.len(), 1);
        assert_eq!(checksums.get("file.tar.gz"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_parse_checksums_empty() {
        let checksums = parse_checksums("");
        assert!(checksums.is_empty());
    }

    #[test]
    fn test_parse_checksums_blank_lines() {
        let content = "\n\nabc123  file.tar.gz\n\n";
        let checksums = parse_checksums(content);

        assert_eq!(checksums.len(), 1);
        assert_eq!(checksums.get("file.tar.gz"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_parse_checksums_malformed_lines() {
        // Lines without proper format should be skipped
        let content = "invalid_line_no_space\n\
                       abc123  valid.tar.gz\n\
                         \n\
                       also_invalid\n";
        let checksums = parse_checksums(content);

        assert_eq!(checksums.len(), 1);
        assert_eq!(checksums.get("valid.tar.gz"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_verify_checksum_success() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        // SHA256 of "test content"
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

        assert!(Downloader::verify_checksum(file.path(), expected).is_ok());
    }

    #[test]
    fn test_verify_checksum_failure() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        let result = Downloader::verify_checksum(file.path(), "wronghash");

        assert!(result.is_err());
        match result {
            Err(DownloadError::ChecksumMismatch { expected, actual }) => {
                assert_eq!(expected, "wronghash");
                assert_eq!(
                    actual,
                    "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
                );
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[test]
    fn test_verify_checksum_empty_file() {
        let file = NamedTempFile::new().unwrap();

        // SHA256 of empty file
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert!(Downloader::verify_checksum(file.path(), expected).is_ok());
    }
}
