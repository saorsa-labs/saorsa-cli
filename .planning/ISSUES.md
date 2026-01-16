# ISSUES.md - Deferred Work Backlog

## P0: Blockers (Immediate)

None currently.

---

## P1: Next Phase

### Windows Build Support
**Context**: Windows build disabled in release.yml due to vendored OpenSSL compilation issues
**Impact**: Windows users must use `cargo install sb` instead of binary download
**Resolution**: Investigate native TLS on Windows or fix vendored OpenSSL build

### GH_RELEASE_TOKEN
**Context**: Custom token had bad credentials, using default GITHUB_TOKEN now
**Impact**: Works but may have different rate limits
**Resolution**: Verify token permissions if issues arise

---

## P2: This Milestone

### Prerelease Channel Support
**Context**: Interview decided stable-only, but config has `check_prerelease` field
**Impact**: Feature exists in config but not wired up
**Resolution**: Wire up prerelease checking in Phase 3

### Error Recovery
**Context**: What happens if update download fails mid-stream?
**Impact**: Could leave partial files
**Resolution**: Implement atomic download (temp file, rename on complete)

---

## P3: Future

### GPG Signature Verification
**Context**: SHA256 checksums chosen over GPG for simplicity
**Impact**: Lower security than full cryptographic signing
**Resolution**: Could add as optional enhanced security feature

### Delta Updates
**Context**: Currently downloads full binary archives each time
**Impact**: Larger downloads than necessary
**Resolution**: Investigate bsdiff or similar for incremental updates

### Update Scheduling
**Context**: Interview decided on-startup checking
**Impact**: Users who never restart CLI won't see updates
**Resolution**: Could add optional interval-based checking

### Offline Mode
**Context**: No graceful handling of network unavailability
**Impact**: Update checks could show errors in offline environments
**Resolution**: Detect offline state, skip checks gracefully

---

## Resolved

| Issue | Resolution | Date |
|-------|------------|------|
| - | - | - |
