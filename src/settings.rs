use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const PREFIXES_DIR_NAME: &str = "Prefixes";

const SETTINGS_FILE_NAME: &str = "com.slawomirkaczmarek.cider.plist";

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Properties {
    pub default_prefix: Option<String>,
    pub prefixes: HashMap<String, PrefixProperties>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PrefixProperties {
    pub custom_dir: Option<String>,
    pub hud: bool,
    pub esync: bool,
    pub retina_mode: bool,
    pub avx: bool,
    pub dxr: bool,
    pub metalfx: bool,
}

macro_rules! settings_file {
    () => {
        (app_support_dir()?).join(SETTINGS_FILE_NAME)
    };
}

pub fn app_support_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("No home directory defined for the current user")?;
    let app_support_dir = home_dir.join("Library/Application Support/Cider");
    if !app_support_dir.exists() {
        let prefixes_dir = app_support_dir.join(PREFIXES_DIR_NAME);
        std::fs::create_dir_all(&prefixes_dir)?;
    }
    Ok(app_support_dir)
}

pub fn open() -> Result<Properties> {
    let settings_file = settings_file!();
    let settings = if settings_file.exists() {
        plist::from_file(&settings_file)?
    } else {
        Properties::default()
    };
    Ok(settings)
}

pub fn save(settings: &Properties) -> Result<()> {
    let settings_file = settings_file!();
    plist::to_file_binary(&settings_file, &settings).context("Unable to save settings")
}
