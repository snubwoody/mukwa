// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub struct Manifest{
    releases: Vec<Release>,
}

#[derive(Clone,Copy,Hash,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
pub enum Platform{
    WindowsX86_64Exe,
    LinuxX86_64AppImage,
}

#[derive(PartialEq,Serialize,Deserialize)]
pub struct Release{
    version: String,
    pub_date: jiff::Zoned,
    artifacts: HashMap<Platform, Artifact>,
}

#[derive(PartialEq,PartialOrd,Serialize,Deserialize)]
pub struct Artifact{
    download_url: String,
    hash: String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReleaseResponse{
    releases: Vec<ReleaseJson>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReleaseJson{
    immutable: bool,
    name: String,
    prerelease: bool,
    draft: bool,
    published_at: String,
    updated_at: String,
    url: String,
    created_at: String,
    body: String,
    assets_url: String,
    assets: Vec<ReleaseAsset>
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReleaseAsset{
    browser_download_url: String,
    content_type: String,
    created_at: String,
    updated_at: String,
    url: String,
    digest: String,
    name: String,
    size: u64
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn gen_release_info() -> crate::Result<()>{
        let response = ureq::get("https://api.github.com/repos/snubwoody/mukwa/releases")
            .call()
            .unwrap()
            .body_mut()
            .read_json::<Vec<ReleaseJson>>()
            .unwrap();
        dbg!(response);
        Ok(())
    }
}