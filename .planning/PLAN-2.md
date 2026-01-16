# PLAN-2: Checksum Generation

**Phase**: 2 of 5
**Milestone**: M1 - Auto-Update Foundation
**Created**: 2026-01-16

## Overview

Implement SHA256 checksum generation and verification for secure binary downloads:
- Generate checksums during release workflow
- Include checksums in release artifacts
- Parse checksums from releases
- Verify downloaded files before extraction

## Prerequisites

- [x] Phase 1 completed (version infrastructure)
- [x] Release workflow already produces binary archives
- [x] downloader.rs exists with download/extraction logic

## Current State

| Component | Status |
|-----------|--------|
| release.yml | Produces archives, no checksums |
| downloader.rs | Downloads work, no verification |
| sha2/hex crates | Already in cli/Cargo.toml |

## Tasks

<task type="auto" priority="p0">
  <n>Add SHA256 checksum generation to release workflow</n>
  <files>
    .github/workflows/release.yml
  </files>
  <action>
    1. Update the `github-release` job to generate checksums after flattening:
       ```yaml
       - name: Generate checksums
         run: |
           cd release-files
           sha256sum * > CHECKSUMS.txt
           cat CHECKSUMS.txt
       ```

    2. The CHECKSUMS.txt format will be standard sha256sum output:
       ```
       abc123...  saorsa-cli-x86_64-apple-darwin.tar.gz
       def456...  saorsa-cli-aarch64-apple-darwin.tar.gz
       ...
       ```

    3. Update the `Create Release` step to include CHECKSUMS.txt:
       The `files: release-files/*` glob already includes all files,
       so CHECKSUMS.txt will be uploaded automatically.

    4. Test by examining a release to confirm CHECKSUMS.txt is present
  </action>
  <verify>
    # Validate YAML syntax
    python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"

    # Or use actionlint if available
    actionlint .github/workflows/release.yml || true
  </verify>
  <done>
    - release.yml includes checksum generation step
    - CHECKSUMS.txt uploaded as release artifact
    - Workflow YAML is valid
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add checksum parsing and verification to downloader</n>
  <files>
    cli/src/downloader.rs
  </files>
  <action>
    1. Add DownloadError variant for checksum failures:
       ```rust
       #[error("Checksum verification failed: expected {expected}, got {actual}")]
       ChecksumMismatch { expected: String, actual: String },

       #[error("Checksum not found for asset: {0}")]
       ChecksumNotFound(String),
       ```

    2. Add GitHubRelease field for body (contains checksums in notes):
       ```rust
       pub struct GitHubRelease {
           pub tag_name: String,
           pub name: Option<String>,
           pub assets: Vec<GitHubAsset>,
           pub published_at: String,
           pub body: Option<String>,  // Release notes (may contain checksums)
       }
       ```

    3. Add checksum-related methods to Downloader:
       ```rust
       /// Fetch the CHECKSUMS.txt asset content from a release
       async fn fetch_checksums(&self, release: &GitHubRelease) -> Result<HashMap<String, String>> {
           // Look for CHECKSUMS.txt asset
           let checksums_asset = release.assets.iter()
               .find(|a| a.name == "CHECKSUMS.txt")
               .ok_or(DownloadError::ChecksumNotFound("CHECKSUMS.txt".to_string()))?;

           // Download and parse
           let content = self.client
               .get(&checksums_asset.browser_download_url)
               .send().await?
               .text().await?;

           Ok(parse_checksums(&content))
       }

       /// Parse sha256sum format: "hash  filename"
       fn parse_checksums(content: &str) -> HashMap<String, String> {
           content.lines()
               .filter_map(|line| {
                   let parts: Vec<&str> = line.split_whitespace().collect();
                   if parts.len() >= 2 {
                       Some((parts[1].to_string(), parts[0].to_string()))
                   } else {
                       None
                   }
               })
               .collect()
       }

       /// Verify a file's SHA256 checksum
       fn verify_checksum(path: &Path, expected: &str) -> Result<(), DownloadError> {
           use sha2::{Sha256, Digest};

           let mut file = File::open(path)?;
           let mut hasher = Sha256::new();
           io::copy(&mut file, &mut hasher)?;
           let actual = hex::encode(hasher.finalize());

           if actual != expected {
               return Err(DownloadError::ChecksumMismatch {
                   expected: expected.to_string(),
                   actual,
               });
           }
           Ok(())
       }
       ```

    4. Update download_binary() to verify after download:
       ```rust
       // After download_asset()
       if let Ok(checksums) = self.fetch_checksums(&release).await {
           if let Some(expected) = checksums.get(&asset.name) {
               Self::verify_checksum(&archive_path, expected)?;
               tracing::info!("Checksum verified for {}", asset.name);
           }
       } else {
           tracing::warn!("No checksums available, skipping verification");
       }
       ```

    5. Make verification optional but log warnings when unavailable
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo test -p cli
  </verify>
  <done>
    - DownloadError has checksum variants
    - fetch_checksums() retrieves CHECKSUMS.txt
    - parse_checksums() handles sha256sum format
    - verify_checksum() computes and compares SHA256
    - download_binary() verifies when checksums available
    - Graceful fallback when checksums unavailable
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p2">
  <n>Add checksum verification tests</n>
  <files>
    cli/src/downloader.rs
  </files>
  <action>
    1. Add unit tests for checksum parsing:
       ```rust
       #[cfg(test)]
       mod tests {
           use super::*;

           #[test]
           fn test_parse_checksums() {
               let content = "abc123def456  file1.tar.gz\n789xyz000111  file2.tar.gz\n";
               let checksums = parse_checksums(content);
               assert_eq!(checksums.get("file1.tar.gz"), Some(&"abc123def456".to_string()));
               assert_eq!(checksums.get("file2.tar.gz"), Some(&"789xyz000111".to_string()));
           }

           #[test]
           fn test_parse_checksums_empty() {
               let checksums = parse_checksums("");
               assert!(checksums.is_empty());
           }

           #[test]
           fn test_parse_checksums_malformed() {
               let content = "invalid line\n  \nabc123  file.tar.gz\n";
               let checksums = parse_checksums(content);
               assert_eq!(checksums.len(), 1);
               assert_eq!(checksums.get("file.tar.gz"), Some(&"abc123".to_string()));
           }
       }
       ```

    2. Add integration test for checksum verification (using tempfile):
       ```rust
       #[test]
       fn test_verify_checksum() {
           use std::io::Write;
           use tempfile::NamedTempFile;

           let mut file = NamedTempFile::new().unwrap();
           file.write_all(b"test content").unwrap();

           // SHA256 of "test content"
           let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

           assert!(verify_checksum(file.path(), expected).is_ok());
           assert!(verify_checksum(file.path(), "wronghash").is_err());
       }
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo test -p cli -- --nocapture
  </verify>
  <done>
    - parse_checksums() has comprehensive tests
    - verify_checksum() tested with real file hashing
    - Edge cases covered (empty, malformed)
    - All tests pass
  </done>
</task>

## Exit Criteria

- [ ] release.yml generates CHECKSUMS.txt with SHA256 hashes
- [ ] CHECKSUMS.txt is uploaded as release artifact
- [ ] downloader.rs can fetch and parse checksums
- [ ] Downloaded archives are verified against checksums
- [ ] Graceful fallback when checksums unavailable
- [ ] All tests pass
- [ ] Zero clippy warnings

## Notes

- sha256sum format: `<hash>  <filename>` (two spaces between hash and filename)
- Using sha2 and hex crates (already in cli/Cargo.toml)
- Verification is optional to support older releases without checksums
- Logging warnings for missing checksums aids debugging

## Security Considerations

- CHECKSUMS.txt is downloaded from the same GitHub release as binaries
- This provides integrity verification, not origin authentication
- For full security, code signing (already done for macOS) is the trust anchor
- Checksum verification catches download corruption and tampering in transit

## Next Phase

Phase 3: Update Checker
- Background async update check
- Cache results with 1-hour TTL
- Store check timestamp in config
