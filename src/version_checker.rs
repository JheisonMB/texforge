//! GitHub releases version checker for auto-update detection.
//!
//! Queries the GitHub API to detect new stable versions and compare with local version.

use crate::version::SemVer;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

/// GitHub API release response (minimal fields)
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    prerelease: bool,
    draft: bool,
}

/// Version check result
#[derive(Debug, Clone)]
pub struct VersionCheckResult {
    pub local_version: SemVer,
    pub latest_stable: Option<SemVer>,
    pub update_available: bool,
}

/// Check for newer stable versions on GitHub
pub fn check_for_updates(owner: &str, repo: &str) -> Result<VersionCheckResult> {
    let local = get_local_version()?;

    // Query GitHub API for releases
    let latest_stable = fetch_latest_stable_release(owner, repo)?;

    let update_available = if let Some(ref remote) = latest_stable {
        remote > &local
    } else {
        false
    };

    Ok(VersionCheckResult {
        local_version: local,
        latest_stable,
        update_available,
    })
}

/// Get the current texforge version (from Cargo.toml at compile time)
pub fn get_local_version() -> Result<SemVer> {
    let version_str = env!("CARGO_PKG_VERSION");
    SemVer::parse(version_str)
        .ok_or_else(|| anyhow!("Failed to parse local version: {}", version_str))
}

/// Fetch latest stable release from GitHub
/// Filters out pre-releases and drafts
fn fetch_latest_stable_release(owner: &str, repo: &str) -> Result<Option<SemVer>> {
    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "texforge")
        .send()
        .context("Failed to query GitHub API")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API returned status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .context("Failed to parse GitHub releases JSON")?;

    // Find the latest stable version (skip pre-releases and drafts)
    for release in releases {
        if !release.draft && !release.prerelease {
            // Remove 'v' prefix if present
            let tag = release.tag_name.trim_start_matches('v');
            if let Some(version) = SemVer::parse(tag) {
                return Ok(Some(version));
            }
        }
    }

    Ok(None)
}

/// Get the download URL for a specific release
pub fn get_release_download_url(owner: &str, repo: &str, version: &SemVer) -> String {
    let arch = get_architecture();
    let _os = get_os();
    let filename = format!("{}-{}-{}", repo, version, arch);

    format!(
        "https://github.com/{}/{}/releases/download/v{}/{}",
        owner, repo, version, filename
    )
}

fn get_architecture() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    return "x86_64";
    #[cfg(target_arch = "aarch64")]
    return "aarch64";
    #[cfg(target_arch = "arm")]
    return "arm";
}

fn get_os() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_version() {
        let version = get_local_version().unwrap();
        assert!(version.major >= 0);
    }
}
