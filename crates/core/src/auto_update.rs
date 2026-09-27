// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use jiff::Timestamp;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct Manifest {
    releases: Vec<Release>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "windows_x86_64_exe")]
    WindowsX86_64Exe,
    #[serde(rename = "windows_aarch64_exe")]
    WindowsAarch64Exe,
    #[serde(rename = "linux_x86_64_appimage")]
    LinuxX86_64AppImage,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Release {
    version: Version,
    published_at: Timestamp,
    artifacts: HashMap<Platform, Artifact>,
    prerelease: bool,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Artifact {
    download_url: String,
    digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResponse {
    releases: Vec<ReleaseJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseJson {
    immutable: bool,
    name: String,
    prerelease: bool,
    draft: bool,
    published_at: Timestamp,
    updated_at: Timestamp,
    url: String,
    created_at: Timestamp,
    tag_name: String,
    body: String,
    assets_url: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAsset {
    browser_download_url: String,
    content_type: String,
    created_at: String,
    updated_at: String,
    url: String,
    digest: String,
    name: String,
    size: u64,
}

/// Generates a release manifest from a list of Github releases.
pub fn gen_release_manifest(releases: &[ReleaseJson]) -> crate::Result<Manifest> {
    let mut manifest = Manifest::default();
    for release in releases {
        if release.draft {
            continue;
        }

        let tag_name = &release.tag_name.strip_prefix("v").unwrap_or_default();
        let mut release_info = Release {
            version: Version::parse(&tag_name)?,
            published_at: release.published_at,
            artifacts: HashMap::new(),
            prerelease: release.prerelease,
        };

        for asset in &release.assets {
            let mut platform = Platform::WindowsX86_64Exe;
            if asset.name.ends_with("x86_64-Setup.exe") {
                platform = Platform::WindowsX86_64Exe;
            }

            if asset.name.ends_with("aarch64-Setup.exe") {
                platform = Platform::WindowsAarch64Exe;
            }

            if asset.name.ends_with("x86_64.AppImage") {
                platform = Platform::LinuxX86_64AppImage;
            }

            let artifact = Artifact {
                download_url: asset.browser_download_url.clone(),
                digest: asset
                    .digest
                    .strip_prefix("sha256:")
                    .unwrap_or_default()
                    .to_owned(),
            };

            release_info.artifacts.insert(platform, artifact);
        }

        manifest.releases.push(release_info);
    }

    Ok(manifest)
}

pub async fn fetch_releases() -> crate::Result<Vec<ReleaseJson>> {
    let releases = smol::unblock(|| {
        ureq::get("https://api.github.com/repos/snubwoody/mukwa/releases")
            .call()?
            .body_mut()
            .read_json::<Vec<ReleaseJson>>()
    })
    .await?;

    Ok(releases)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gen_release_info() -> crate::Result<()> {
        let releases = smol::block_on(async { fetch_releases().await })?;

        let manifest = gen_release_manifest(&releases)?;
        let content = serde_json::to_string_pretty(&manifest).unwrap();
        std::fs::write("manifest.json", content)?;
        Ok(())
    }
}
