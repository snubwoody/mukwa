// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::Result;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::Read;
use tempfile::tempdir;

use mukwa_core::Money;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use tracing::{error, info,warn};

#[derive(Clone)]
pub struct SettingsStore {
    path: PathBuf,
    inner: Rc<RefCell<Settings>>,
}

impl SettingsStore {
    pub fn set_currency_code(&self, code: &str) -> Result<()> {
        self.settings_mut().currency_code = code.to_owned();
        self.write()?;
        info!("Updated currency code to {code}");
        Ok(())
    }

    pub fn set_font_family(&self, family: &str) -> Result<()> {
        self.settings_mut().appearance.font_family = family.to_owned();
        self.write()?;
        info!("Updated font family to {family}");
        Ok(())
    }

    pub fn currency_code(&self) -> String {
        self.settings().currency_code.clone()
    }

    pub fn font_family(&self) -> String {
        self.settings().appearance.font_family.clone()
    }

    fn settings(&self) -> Ref<'_, Settings> {
        self.inner.borrow()
    }

    fn settings_mut(&self) -> RefMut<'_, Settings> {
        self.inner.borrow_mut()
    }

    fn write(&self) -> Result<()> {
        let settings = self.settings();
        let contents = toml::to_string(&*settings)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }

    pub fn open(path: impl AsRef<Path>) -> Result<SettingsStore> {
        match File::open(&path) {
            Ok(mut file) => {
                info!("Loading settings from {:?}", path.as_ref());

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let settings: Settings = toml::from_slice(&buffer)?;
                let store = SettingsStore {
                    path: path.as_ref().to_path_buf(),
                    inner: Rc::new(RefCell::new(settings)),
                };
                Ok(store)
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(err.into());
                }

                let settings = Settings::default();
                let contents = toml::to_string(&settings)?;
                fs::write(&path, contents)?;

                info!("Initialised settings at {:?}", path.as_ref());
                let store = SettingsStore {
                    path: path.as_ref().to_path_buf(),
                    inner: Rc::new(RefCell::new(settings)),
                };
                Ok(store)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    currency_code: String,
    #[serde(default)]
    appearance: Appearance,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Appearance {
    font_family: String,
}

impl Default for Appearance {
    fn default() -> Self {
        let default_font = if cfg!(target_os = "windows") {
            "Segoe UI"
        } else if cfg!(target_os = "macos") {
            "SF Pro"
        } else {
            // Slint will use the default font
            ""
        };

        Self {
            font_family: String::from(default_font),
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            currency_code: String::from("USD"),
            appearance: Appearance::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;
    use crate::SettingsStore;
    use std::fs;

    #[test]
    fn init_settings_if_not_found() -> Result<()> {
        let temp = tempdir()?;
        let path = temp.path().join("settings.toml");
        SettingsStore::open(&path)?;
        assert!(fs::exists(path)?);
        Ok(())
    }
}
