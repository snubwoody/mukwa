use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReleaseManifest {
    version: String,
    url: String,
}

// TODO: add auto-update setting (bool)
pub struct AutoUpdater {
    /// The directory the update is stored in
    update_dir: PathBuf,
}

impl AutoUpdater {
    pub fn new(path: impl AsRef<Path>) -> AutoUpdater {
        AutoUpdater {
            update_dir: path.as_ref().to_path_buf(),
        }
    }

    /// Checks for updates.
    fn check(&self, url: &str) -> crate::Result<Option<ReleaseManifest>> {
        let response = ureq::get(url).call()?;
        let reader = response.into_body().into_reader();
        // TODO: parsing from a string is faster
        // https://github.com/serde-rs/json/issues/160
        let manifest: ReleaseManifest = serde_json::from_reader(reader)?;
        Ok(Some(manifest))
    }

    /// Downloads the latest update into the `update_dir` directory.
    fn download_update(&self, url: &str) -> crate::Result<()> {
        let response = ureq::get(url).call()?;
        let mut reader = response.into_body().into_reader();

        // TODO: stream directly instead of loading into memory
        let mut buffer: Vec<u8> = Vec::new();

        reader.read_to_end(&mut buffer)?;
        let path = self.update_dir.join("Mukwa-Update.exe");
        fs::write(&path, &buffer)?;
        // TODO: might need to return PathBuf for logging
        // TODO: delete after updating
        Ok(())
    }
}

/// Downloads the latest update into the local app data directory.
fn download_update(url: &str) -> crate::Result<()> {
    let dir = dirs::data_local_dir().unwrap().join("Mukwa");
    Ok(())
}

pub fn install_update() -> crate::Result<()> {
    // TODO: download into localappdata/updates dir
    let url = "https://github.com/snubwoody/mukwa/releases/download/v0.1.0-alpha.4/Mukwa-x86_64-Setup.exe";
    let response = ureq::get(url).call()?;
    let mut reader = response.into_body().into_reader();

    // TODO: stream directly instead of loading into memory
    let mut buffer: Vec<u8> = Vec::new();

    reader.read_to_end(&mut buffer)?;
    let path = PathBuf::from("Mukwa-Update.exe");
    fs::write(&path, &buffer)?;
    // TODO: output logs to log dir
    // TODO: test installing the same version and previous versions?
    // TODO: zed had /update=true
    // DOC: brings popup when not installed
    let absolute_path = path.canonicalize()?;
    dbg!(&absolute_path);
    let output = Command::new(absolute_path).arg("/verysilent").output()?;
    assert!(output.status.success());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    // TODO: simulate failures
    #[test]
    fn check_for_new_update() -> crate::Result<()> {
        let server = MockServer::start();

        let release_manifest = ReleaseManifest {
            version: String::from("v0.1.0-alpha.2"),
            url: server.url("/release-manifest.json"),
        };
        let mock = server.mock(|when, then| {
            when.method(GET).path("/release-manifest.json");

            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&release_manifest);
        });

        let temp_dir = tempdir()?;
        let auto_updater = AutoUpdater::new(temp_dir.path());
        let update = auto_updater.check(&server.url("/release-manifest.json"))?;
        mock.assert();

        assert!(update.is_some());
        let update = update.unwrap();
        assert_eq!(update, release_manifest);
        Ok(())
    }

    #[test]
    fn download_update_into_directory() -> crate::Result<()> {
        let url = "https://github.com/snubwoody/mukwa/releases/download/v0.1.0-alpha.4/Mukwa-x86_64-Setup.exe";
        let temp_dir = tempdir()?;
        let auto_updater = AutoUpdater::new(temp_dir.path());
        auto_updater.download_update(&url)?;

        let exists = fs::exists(temp_dir.path().join("Mukwa-Update.exe"))?;
        assert!(exists);
        Ok(())
    }

    #[test]
    fn update() -> crate::Result<()> {
        install_update()?;
        Ok(())
    }
}
