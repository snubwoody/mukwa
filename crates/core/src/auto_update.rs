// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use jiff::Timestamp;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::{log_dir, Error};

#[derive(Serialize, Deserialize, Default)]
pub struct Manifest {
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "windows_x86_64_exe")]
    WindowsX64,
    #[serde(rename = "windows_aarch64_exe")]
    WindowsArm64,
    #[serde(rename = "linux_x86_64_appimage")]
    LinuxX64AppImage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub published_at: Timestamp,
    pub artifacts: HashMap<Platform, Artifact>,
    pub prerelease: bool,
}

impl Default for Release {
    fn default() -> Self {
        Release {
            version: Version::new(0, 0, 0),
            published_at: Timestamp::default(),
            artifacts: HashMap::new(),
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

/// Generates a release manifest from a list of GitHub releases.
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

pub async fn download_update(release: Release, dir: impl AsRef<Path>) -> crate::Result<PathBuf> {
    let artifact = release.artifacts.get(&Platform::WindowsX64).unwrap();
    let download_url = artifact.download_url.clone();
    let response = smol::unblock(move || ureq::get(&download_url).call()).await?;

    if !response.status().is_success() {
        // TODO: read response body
        return Err(Error::new("Response error"))
    }
    let dest = dir.as_ref().join("Mukwa-Update.exe");
    smol::unblock({
        let dest = dest.clone();
        move || {
            let mut file = File::create(dest)?;
            let mut reader = response.into_body().into_reader();
            std::io::copy(&mut reader,&mut file)
        }
    }).await?;
    Ok(dest)
}

pub fn install_update(path: impl AsRef<Path>) -> crate::Result<()>{
    let absolute_path = path.as_ref().canonicalize()?;
    let log_path = log_dir().join("inno-setup.log");
    Command::new(absolute_path).arg("/verysilent").arg(&format!("/log={}",log_path.display())).spawn()?;
    Ok(())
}

pub fn check_for_update(current_version: Version, mut manifest: Manifest) -> Option<Release> {
    if manifest.releases.is_empty() {
        return None;
    }
    manifest
        .releases
        .sort_by(|a, b| a.version.cmp(&b.version).reverse());

    if manifest.releases[0].version > current_version {
        return Some(manifest.releases[0].clone());
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_for_latest_update() -> crate::Result<()> {
        let releases = vec![Release {
            version: Version::new(0, 1, 0),
            ..Default::default()
        }];
        let manifest = Manifest { releases };
        let release = check_for_update(Version::new(0, 0, 0), manifest);
        assert!(release.is_some());
        assert_eq!(release.unwrap().version, Version::new(0, 1, 0));
        Ok(())
    }

    #[test]
    fn latest_version_equals_current_version() -> crate::Result<()> {
        let releases = vec![Release {
            version: Version::new(0, 1, 0),
            ..Default::default()
        }];
        let manifest = Manifest { releases };
        let release = check_for_update(Version::new(0, 1, 0), manifest);
        assert!(release.is_none());
        Ok(())
    }

    #[test]
    fn latest_version_is_less_than_current_version() -> crate::Result<()> {
        let releases = vec![Release {
            version: Version::new(0, 0, 0),
            ..Default::default()
        }];
        let manifest = Manifest { releases };
        let release = check_for_update(Version::new(0, 1, 0), manifest);
        assert!(release.is_none());
        Ok(())
    }

    #[test]
    fn check_for_update_returns_newest_update() -> crate::Result<()> {
        let releases = vec![
            Release {
                version: Version::new(0, 1, 0),
                ..Default::default()
            },
            Release {
                version: Version::new(0, 2, 0),
                ..Default::default()
            },
            Release {
                version: Version::new(1, 0, 0),
                ..Default::default()
            },
        ];
        let manifest = Manifest { releases };
        let release = check_for_update(Version::new(0, 0, 0), manifest);
        assert!(release.is_some());
        assert_eq!(release.unwrap().version, Version::new(1, 0, 0));
        Ok(())
    }
}
