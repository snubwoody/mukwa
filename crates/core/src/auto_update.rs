// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use jiff::Timestamp;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Manifest {
    pub releases: Vec<Release>,
}

impl Manifest {
    pub fn create() -> crate::Result<Manifest> {
        let releases = Manifest::fetch_releases()?;
        let manifest = Manifest::parse(&releases)?;
        Ok(manifest)
    }

    fn fetch_releases() -> crate::Result<Vec<ReleaseJson>> {
        let releases = ureq::get("https://api.github.com/repos/snubwoody/mukwa/releases")
            .call()?
            .body_mut()
            .read_json::<Vec<ReleaseJson>>()?;

        Ok(releases)
    }

    /// Creates a release manifest from a list of GitHub releases.
    fn parse(releases: &[ReleaseJson]) -> crate::Result<Manifest> {
        let mut manifest = Manifest::default();
        for release in releases {
            if release.draft {
                continue;
            }

            let tag_name = &release.tag_name.strip_prefix("v").unwrap_or_default();
            let mut release_info = Release {
                version: Version::parse(tag_name)?,
                published_at: release.published_at,
                artifacts: BTreeMap::new(),
                prerelease: release.prerelease,
            };

            for asset in &release.assets {
                let mut platform = Platform::WindowsX64;
                if asset.name.ends_with("x86_64-Setup.exe") {
                    platform = Platform::WindowsX64;
                }

                if asset.name.ends_with("aarch64-Setup.exe") {
                    platform = Platform::WindowsArm64;
                }

                if asset.name.ends_with("x86_64.AppImage") {
                    platform = Platform::LinuxX64AppImage;
                }

                if asset.name.ends_with("x86_64.deb") {
                    platform = Platform::LinuxX64Deb;
                }

                if asset.name.ends_with("x86_64.tar.gz") {
                    platform = Platform::LinuxX64Tarball;
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
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "windows_x86_64_exe")]
    WindowsX64,
    #[serde(rename = "windows_aarch64_exe")]
    WindowsArm64,
    #[serde(rename = "linux_x86_64_appimage")]
    LinuxX64AppImage,
    #[serde(rename = "linux_x86_64_tarball")]
    LinuxX64Tarball,
    #[serde(rename = "linux_x86_64_deb")]
    LinuxX64Deb,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub published_at: Timestamp,
    pub artifacts: BTreeMap<Platform, Artifact>,
    pub prerelease: bool,
}

impl Default for Release {
    fn default() -> Self {
        Release {
            version: Version::new(0, 0, 0),
            published_at: Timestamp::default(),
            artifacts: BTreeMap::new(),
            prerelease: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Artifact {
    pub download_url: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResponse {
    pub releases: Vec<ReleaseJson>,
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
