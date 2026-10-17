// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use semver::Version;
use slint::{ComponentHandle, Global};
use tracing::{debug, info, warn};
use mukwa_core::auto_update::{download_update, Manifest, install_update};
use mukwa_core::error::ErrorExt;
use crate::ui::{self, MainWindow, AutoUpdater, AutoUpdaterState};

pub fn bind(app: &MainWindow) -> crate::Result<()>{
    AutoUpdateController::bind(app)?;
    Ok(())
}

// TODO:
// - Check for existing update
// - Validate checksum
// - Store update path
#[derive(Clone)]
pub struct AutoUpdateController{
    updater: Rc<AutoUpdater<'static>>,
    update_dir: PathBuf,
    current_version: Version
}

impl AutoUpdateController{
    fn bind(app: &MainWindow) -> crate::Result<Self>{
        let update_dir = dirs::data_local_dir()
            .unwrap()
            .join("Mukwa")
            .join("updates");

        let updater: AutoUpdater = app.global();
        let controller = Self {
            updater: Rc::new(updater.as_weak().unwrap()),
            update_dir,
            current_version: Version::parse(env!("CARGO_PKG_VERSION"))?
        };

        #[cfg(not(debug_assertions))]
        let _ = slint::spawn_local({
            let controller = controller.clone();
            async move {
                // Give the app some time to set up the state and callbacks
                smol::Timer::after(Duration::from_secs(10)).await;
                let manifest_url =
                    "https://raw.githubusercontent.com/snubwoody/mukwa/refs/heads/main/release-manifest.json";
                if let Err(err) = controller.check_for_update(manifest_url)
                    .await
                    .context("Failed to check for update")
                {
                    warn!("{}", err.report());
                }
            }
        });

        updater.on_update_and_restart({
            let controller = controller.clone();
            move || {
                if let Err(err) = controller.update_and_restart().context("Failed to update app") {
                    warn!("{}", err.report())
                }
            }
        });
        Ok(controller)
    }

    fn update_and_restart(&self) -> crate::Result<()>{
        install_update(self.update_dir.join("Mukwa-Update.exe"))
    }

    async fn check_for_update(&self,url: &str) -> crate::Result<()> {
        self.updater.set_update_state(AutoUpdaterState::Idle);
        let result = smol::unblock({
            let url = url.to_owned();
            move || ureq::get(url).call()
        }).await;

        match result {
            Ok(response) => {
                // TODO: check if this blocks
                let reader = response.into_body().into_reader();
                let manifest: Manifest = serde_json::from_reader(reader)?;
                match mukwa_core::auto_update::check_for_update(self.current_version.clone(),manifest){
                    Some(release) => {
                        info!("New update found, version {}", release.version);
                        self.updater.set_update_state(AutoUpdaterState::Downloading);
                        info!("Downloading new update...");
                        smol::fs::create_dir_all(&self.update_dir).await?;
                        let update_path = download_update(release, &self.update_dir).await?;
                        debug!(path=?update_path,"Successfully downloaded new update");
                        self.updater.set_update_state(AutoUpdaterState::Ready);
                    },
                    None => {
                        self.updater.set_update_state(AutoUpdaterState::Idle);
                        info!("No new update found");
                    }
                }
            }
            Err(err) => {
                self.updater.set_update_state(AutoUpdaterState::Idle);
                warn!("Failed to fetch manifest: {}",err)
            }
        }
        Ok(())
    }

}

// TODO:
// test - partially downloaded file
// test - already downloaded file
// test - windows release artifact not found
// TODO: only run on Windows
#[cfg(test)]
mod test{
    use std::collections::HashMap;
    use std::fs;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use semver::Version;
    use tempfile::tempdir;
    use mukwa_core::auto_update::{Artifact, Manifest, Platform, Release};
    use super::*;
    use crate::ui::MainWindow;

    #[test]
    fn manifest_url_not_found(){
        smol::block_on(async {
            i_slint_backend_testing::init_no_event_loop();
            let app = MainWindow::new().unwrap();
            let controller = AutoUpdateController::bind(&app).unwrap();
            controller.check_for_update("https://oijoijoij.c").await.unwrap();
        })
    }

    #[test]
    fn fetch_and_download_update() -> crate::Result<()>{
        let server = MockServer::start();

        let download_mock = server.mock(move |when,then|{
            when.method(GET)
                .path("/Mukwa-x86_64-Setup.exe");

            then
                .status(200)
                .body(vec![20;20]);
        });

        let mut artifacts = HashMap::new();
        artifacts.insert(
            Platform::WindowsX64,
            Artifact {
                download_url: server.url("/Mukwa-x86_64-Setup.exe"),
                digest: String::new(),
            },
        );

        let release = Release{
            version: Version::new(0,5,0),
            published_at: Default::default(),
            artifacts,
            prerelease: false,
        };

        let temp = tempdir()?;
        let manifest = Manifest{
            releases: vec![release]
        };

        let mock = server.mock(move |when,then|{
            when.method(GET)
                .path("/release-manifest.json");

            then
                .status(200)
                .header("content-type","application/json")
                .json_body_obj(&manifest);
        });

        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let mut controller = AutoUpdateController::bind(&app)?;
        controller.update_dir = temp.path().to_owned();
        controller.current_version = Version::new(0,1,1);
        smol::block_on(async {
            controller.check_for_update(server.url("/release-manifest.json").as_str()).await.unwrap();
        });

        mock.assert();
        download_mock.assert();
        assert_eq!(controller.updater.get_update_state(),AutoUpdaterState::Ready);
        assert!(fs::exists(temp.path().join("Mukwa-Update.exe"))?);
        Ok(())
    }
}