// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{AutoUpdater, AutoUpdaterState, MainWindow};
use mukwa_core::auto_update::{Manifest, Platform, Release};
use mukwa_core::error::ErrorExt;
use mukwa_core::{Error, log_dir};
use semver::Version;
use slint::{ComponentHandle, Global};
use std::process::Command;
use std::time::Duration;
use std::{path::PathBuf, rc::Rc};
use tracing::{debug, info, warn};

pub fn bind(app: &MainWindow) -> crate::Result<()> {
    AutoUpdateController::bind(app)?;
    Ok(())
}

#[derive(Clone)]
struct AutoUpdateController {
    updater: Rc<AutoUpdater<'static>>,
    current_version: Version,
    update_dir: PathBuf,
    manifest_url: String,
}

impl AutoUpdateController {
    fn bind(app: &MainWindow) -> crate::Result<Self> {
        let update_dir = dirs::data_local_dir()
            .unwrap()
            .join("Mukwa")
            .join("updates");

        let updater: AutoUpdater = app.global();
        let controller = Self {
            updater: Rc::new(updater.as_weak().unwrap()),
            update_dir,
            current_version: Version::parse(env!("CARGO_PKG_VERSION"))?,
            manifest_url: String::from(
                "https://raw.githubusercontent.com/snubwoody/mukwa/refs/heads/main/release-manifest.json",
            ),
        };

        updater.on_update_and_restart({
            let controller = controller.clone();
            move || {
                if let Err(err) = controller.install_update().context("Failed to update app") {
                    warn!("{}", err.report())
                }
            }
        });

        let _ = slint::spawn_local({
            let controller = controller.clone();
            async move {
                if cfg!(debug_assertions) {
                    return;
                }

                // Give the app some time to set up the state and callbacks
                smol::Timer::after(Duration::from_secs(10)).await;
                if let Err(err) = controller
                    .poll()
                    .await
                    .context("Failed to poll for updates")
                {
                    warn!("{}", err.report());
                }
            }
        });

        Ok(controller)
    }

    async fn poll(&self) -> crate::Result<()> {
        loop {
            if let Some(release) = self.check_for_update().await? {
                info!("Found new update, version {}", release.version);
                let path = self.download_update(release).await?;
                debug!(path=?path,"Successfully downloaded new update");
                self.updater.set_update_state(AutoUpdaterState::Ready);
            } else {
                self.updater.set_update_state(AutoUpdaterState::Idle);
                info!("No update found");
            }

            smol::Timer::after(Duration::from_hours(1)).await;
        }
    }

    fn install_update(&self) -> crate::Result<()> {
        // FIXME: store the update path?
        let path = self.update_dir.join("Mukwa-Update.exe");
        let absolute_path = path.canonicalize()?;
        let log_path = log_dir().join("inno-setup.log");
        Command::new(absolute_path)
            .arg("/verysilent")
            .arg("/update=1")
            .arg(format!("/log={}", log_path.display()))
            .spawn()?;
        // TODO: maybe use cleaner exit method to run destructors
        // TODO: use never type when stabilized
        std::process::exit(0);
    }

    async fn download_update(&self, release: Release) -> crate::Result<PathBuf> {
        #[cfg(not(target_os = "windows"))]
        return Err(Error::new("Platform does not support auto updating"));

        #[cfg(target_arch = "x86_64")]
        let platform = Platform::WindowsX64;
        #[cfg(target_arch = "aarch64")]
        let platform = Platform::WindowsArm64;

        let artifact = release
            .artifacts
            .get(&platform)
            .ok_or(Error::new("Release artifact not found"))?;
        let download_url = artifact.download_url.clone();
        info!("Downloading update...");
        self.updater.set_update_state(AutoUpdaterState::Downloading);
        let response = smol::unblock(move || ureq::get(&download_url).call()).await?;

        if !response.status().is_success() {
            // TODO: read response body
            return Err(Error::new("Response error"));
        }
        let dest = self.update_dir.join("Mukwa-Update.exe");
        smol::fs::create_dir_all(&self.update_dir).await?;
        // TODO: use smol::fs instead
        smol::unblock({
            let dest = dest.clone();
            move || {
                let mut file = std::fs::File::create(dest)?;
                let mut reader = response.into_body().into_reader();
                std::io::copy(&mut reader, &mut file)
            }
        })
        .await?;
        Ok(dest)
    }

    async fn check_for_update(&self) -> crate::Result<Option<Release>> {
        info!("Checking for update...");
        let response = ureq::get(&self.manifest_url).call()?;
        let reader = response.into_body().into_reader();
        let mut manifest: Manifest = serde_json::from_reader(reader)?;

        // TODO: might be redundant
        if manifest.releases.is_empty() {
            return Ok(None);
        }
        manifest
            .releases
            .sort_by(|a, b| a.version.cmp(&b.version).reverse());

        if manifest.releases[0].version > self.current_version {
            return Ok(Some(manifest.releases[0].clone()));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use mukwa_core::auto_update::Artifact;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn download_update() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let mut controller = AutoUpdateController::bind(&app)?;
        let temp = tempdir()?;
        controller.update_dir = temp.path().to_owned();

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/Mukwa-x86_64-Setup.exe");

            then.status(200).body(vec![20; 20]);
        });

        let mut release = Release {
            version: Version::new(0, 0, 0),
            published_at: Default::default(),
            artifacts: Default::default(),
            prerelease: false,
        };

        let artifact = Artifact {
            download_url: server.url("/Mukwa-x86_64-Setup.exe"),
            digest: String::new(),
        };

        let platform = if cfg!(target_arch = "aarch64") {
            Platform::WindowsArm64
        } else {
            Platform::WindowsX64
        };

        release.artifacts.insert(platform, artifact);

        let result = smol::block_on(async { controller.download_update(release).await });

        #[cfg(not(target_os = "windows"))]
        {
            assert!(result.is_err());
            return Ok(());
        }

        let update_path = result?;
        mock.assert();
        assert!(fs::exists(&update_path)?);
        let bytes = fs::read(update_path)?;
        assert_eq!(bytes, vec![20; 20]);
        assert_eq!(
            controller.updater.get_update_state(),
            AutoUpdaterState::Downloading
        );
        Ok(())
    }

    #[test]
    fn release_artifact_not_found() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let mut controller = AutoUpdateController::bind(&app)?;
        let temp = tempdir()?;
        controller.update_dir = temp.path().to_owned();

        let mut release = Release {
            version: Version::new(0, 0, 0),
            published_at: Default::default(),
            artifacts: Default::default(),
            prerelease: false,
        };

        let artifact = Artifact {
            download_url: String::new(),
            digest: String::new(),
        };
        release
            .artifacts
            .insert(Platform::LinuxX64AppImage, artifact);

        let result = smol::block_on(async { controller.download_update(release).await });
        assert!(result.is_err());
        Ok(())
    }
}
