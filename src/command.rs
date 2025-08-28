use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Context, Result};

use crate::settings::{PrefixProperties, Properties};

const GPTK_APP_FILE_NAME: &str = "Game Porting Toolkit.app";

const WINE_EXECUTABLE_PATH: &str = "Contents/Resources/wine/bin/wine64";

const CUSTOM_DIR_KEY: &str = "dir";
const HUD_KEY: &str = "hud";
const ESYNC_KEY: &str = "esync";
const RETINA_MODE_KEY: &str = "retina_mode";
const AVX_KEY: &str = "avx";
const DXR_KEY: &str = "dxr";
const METALFX_KEY: &str = "metalfx";

macro_rules! prefix_not_found {
    ($prefix:ident) => {
        format!("Prefix `{}` not found", &$prefix)
    };
}

macro_rules! prefix_properties {
    ($properties:ident, $prefix:ident, $prefix_properties:ident) => {
        let $properties = crate::settings::open()?;
        let $prefix = select_prefix($prefix, &$properties)?;
        let $prefix_properties = $properties.prefixes.get(&$prefix).with_context(|| prefix_not_found!($prefix))?;
    };
}

macro_rules! mut_prefix_properties {
    ($properties:ident, $prefix:ident, $prefix_properties:ident) => {
        let mut $properties = crate::settings::open()?;
        let $prefix = select_prefix($prefix, &$properties)?;
        #[allow(unused_mut)]
        let mut $prefix_properties = $properties.prefixes.get_mut(&$prefix).with_context(|| prefix_not_found!($prefix))?;
    };
}

macro_rules! add_prefix_prelude {
    ($properties:ident, $prefix:ident, $prefix_properties:ident) => {
        let mut $properties = crate::settings::open()?;

        if $properties.prefixes.contains_key(&$prefix) {
            return Err(anyhow!("Prefix `{}` already exists", $prefix))
        }

        let mut $prefix_properties = PrefixProperties::default();
    };
}

macro_rules! std_prefix_path {
    ($prefix:expr) => {
        (crate::settings::app_support_dir()?).join(crate::settings::PREFIXES_DIR_NAME).join($prefix)
    };
}

macro_rules! std_prefix_path_string {
    ($prefix:expr) => {
        std_prefix_path!($prefix).into_os_string().into_string().map_err(|e| anyhow!("Unable to parse prefix path `{:?}`", e))?
    }
}

macro_rules! prefix_path {
    ($prefix_path:ident, $prefix:expr, $prefix_properties:expr) => {
        let $prefix_path = match &$prefix_properties.custom_dir {
            Some(custom_dir) => {
                custom_dir
            },
            None => {
                &std_prefix_path_string!($prefix)
            }
        };
    };
}

macro_rules! string_to_bool {
    ($value:ident) => {
        matches!($value.to_lowercase().as_str(), "true" | "yes" | "y" | "1")
    };
}

macro_rules! bool_to_env_value {
    ($value:expr) => {
        if $value { "1" } else { "0" }
    };
}

macro_rules! print_properties {
    ($($property:expr),*) => {
        $(
            println!("{}={}", $property.0, $property.1);
        )*
    }
}

macro_rules! wine_cmd {
    ($prefix_path:expr) => {
        Command::new((crate::settings::app_support_dir()?).join(GPTK_APP_FILE_NAME).join(WINE_EXECUTABLE_PATH).into_os_string().as_os_str())
            .env("WINEPREFIX", $prefix_path)
    };
}

macro_rules! spawn_wine_cmd {
    ($command:expr) => {
        $command.spawn().context("Unable to spawn Wine process, is the Game Porting Toolkit installed?")?
    };
}

fn select_prefix(prefix: Option<String>, properties: &Properties) -> Result<String> {
    if let Some(prefix) = prefix.or(properties.default_prefix.clone()) {
        Ok(prefix)
    } else {
        Err(anyhow!("No prefix specified and no default prefix set"))
    }
}

pub fn add_prefix(prefix: String, dir: String) -> Result<()> {
    add_prefix_prelude!(properties, prefix, prefix_properties);

    if !PathBuf::from(&dir).exists() {
        return Err(anyhow!("Directory `{dir}` not found"))
    }

    prefix_properties.custom_dir = Some(dir);

    properties.prefixes.insert(prefix.clone(), prefix_properties);

    println!("Prefix `{}` added", &prefix);

    crate::settings::save(&properties)
}

pub fn create_prefix(prefix: String, dir: Option<String>) -> Result<()> {
    add_prefix_prelude!(properties, prefix, prefix_properties);

    let prefix_path = match dir {
        Some(dir) => {
            prefix_properties.custom_dir = Some(dir.clone());
            dir
        },
        None => {
            std_prefix_path_string!(&prefix)
        }
    };

    let mut process = spawn_wine_cmd!(wine_cmd!(&prefix_path)
        .arg("winecfg")
    );

    let status = process.wait()?;

    if status.success() {
        println!("Prefix `{}` created", &prefix);
        properties.prefixes.insert(prefix.clone(), prefix_properties);
        crate::settings::save(&properties)
    } else {
        Err(anyhow!("Unable to create prefix: {status}"))
    }
}

pub fn default_prefix(prefix: Option<String>) -> Result<()> {
    let mut properties = crate::settings::open()?;

    match prefix {
        Some(prefix) => {
            if properties.prefixes.contains_key(&prefix) {
                properties.default_prefix = Some(prefix.clone());
                println!("Prefix `{}` set as default", &prefix);
                crate::settings::save(&properties)
            } else {
                Err(anyhow!(prefix_not_found!(prefix)))
            }
        },
        None => {
            match properties.default_prefix {
                Some(default_prefix) => {
                    println!("Default prefix: `{default_prefix}`");
                },
                None => {
                    println!("No default prefix set")
                }
            }
            Ok(())
        },
    }
}

pub fn list_prefixes() -> Result<()> {
    let properties = crate::settings::open()?;

    for prefix in properties.prefixes.keys() {
        println!("{prefix}{}", if properties.default_prefix.as_ref() == Some(prefix) { " (default)" } else { "" });
    }

    Ok(())
}

pub fn remove_prefix(prefix: Option<String>) -> Result<()> {
    mut_prefix_properties!(properties, prefix, prefix_properties);

    let prefix_path = match &prefix_properties.custom_dir {
        Some(custom_dir) => {
            PathBuf::from(custom_dir)
        },
        None => {
            std_prefix_path!(&prefix)
        }
    };

    std::fs::remove_dir_all(&prefix_path).with_context(|| format!("Unable to remove prefix directory `{}`", &prefix_path.display()))?;
    if properties.default_prefix.as_ref() == Some(&prefix) {
        properties.default_prefix = None;
    }
    properties.prefixes.remove(&prefix);

    println!("Prefix `{}` removed", &prefix);

    crate::settings::save(&properties)
}

pub fn prefix_config(prefix: Option<String>, settings: Vec<(String, String)>) -> Result<()> {
    if settings.is_empty() {
        prefix_properties!(properties, prefix, prefix_properties);

        println!("Prefix `{}` properties:", &prefix);
        print_properties!(
            (CUSTOM_DIR_KEY, prefix_properties.custom_dir.as_deref().unwrap_or(format!(" # {} (default)", std_prefix_path!(&prefix).display()).as_str())),
            (HUD_KEY, prefix_properties.hud),
            (ESYNC_KEY, prefix_properties.esync),
            (RETINA_MODE_KEY, prefix_properties.retina_mode),
            (AVX_KEY, prefix_properties.avx),
            (DXR_KEY, prefix_properties.dxr),
            (METALFX_KEY, prefix_properties.metalfx)
        );

        Ok(())
    } else {
        mut_prefix_properties!(properties, prefix, prefix_properties);

        let mut set_retina_mode = false;

        for (key, value) in settings {
            match key.as_str() {
                CUSTOM_DIR_KEY => {
                    prefix_properties.custom_dir = (!value.is_empty()).then_some(value);
                    print_properties!((CUSTOM_DIR_KEY, prefix_properties.custom_dir.as_deref().unwrap_or("")));
                },
                HUD_KEY => {
                    prefix_properties.hud = string_to_bool!(value);
                    print_properties!((HUD_KEY, prefix_properties.hud));
                },
                ESYNC_KEY => {
                    prefix_properties.esync = string_to_bool!(value);
                    print_properties!((ESYNC_KEY, prefix_properties.esync));
                },
                RETINA_MODE_KEY => {
                    let new_value = string_to_bool!(value);
                    if prefix_properties.retina_mode != new_value {
                        prefix_properties.retina_mode = new_value;
                        set_retina_mode = true;
                    } else {
                        print_properties!((RETINA_MODE_KEY, prefix_properties.retina_mode));
                    }
                },
                AVX_KEY => {
                    prefix_properties.avx = string_to_bool!(value);
                    print_properties!((AVX_KEY, prefix_properties.avx));
                },
                DXR_KEY => {
                    prefix_properties.dxr = string_to_bool!(value);
                    print_properties!((DXR_KEY, prefix_properties.dxr));
                },
                METALFX_KEY => {
                    prefix_properties.metalfx = string_to_bool!(value);
                    print_properties!((METALFX_KEY, prefix_properties.metalfx));
                },
                _ => {
                    return Err(anyhow!("Unsupported property `{}`", key));
                },
            }
        }

        if set_retina_mode {
            prefix_path!(prefix_path, prefix, prefix_properties);

            let mut process = wine_cmd!(prefix_path)
                .arg("reg")
                .arg("add")
                .arg("HKCU\\Software\\Wine\\Mac Driver")
                .arg("/v")
                .arg("RetinaMode")
                .arg("/t")
                .arg("REG_SZ")
                .arg("/d")
                .arg(if prefix_properties.retina_mode { "Y" } else { "N" })
                .arg("/f")
                .spawn()?;

            let status = process.wait()?;

            if !status.success() {
                println!("Unable to set Retina mode: {status}");
                prefix_properties.retina_mode = !prefix_properties.retina_mode;
            }

            print_properties!((RETINA_MODE_KEY, prefix_properties.retina_mode));
        }

        crate::settings::save(&properties)
    }
}

pub fn run(command: String, prefix: Option<String>, args: Vec<String>) -> Result<()> {
    prefix_properties!(properties, prefix, prefix_properties);

    prefix_path!(prefix_path, prefix, prefix_properties);

    let mut process = spawn_wine_cmd!(wine_cmd!(prefix_path)
        .env("MTL_HUD_ENABLED", bool_to_env_value!(prefix_properties.hud))
        .env("WINEESYNC", bool_to_env_value!(prefix_properties.esync))
        .env("ROSETTA_ADVERTISE_AVX", bool_to_env_value!(prefix_properties.avx))
        .env("D3DM_SUPPORT_DXR", bool_to_env_value!(prefix_properties.dxr))
        .env("D3DM_ENABLE_METALFX", bool_to_env_value!(prefix_properties.metalfx))
        .arg(command)
        .args(args)
    );

    let status = process.wait()?;

    if !status.success() {
        println!("Unable to run command: {status}");
    }

    Ok(())
}

pub fn install(path: String) -> Result<()> {
    let from = PathBuf::from(&path);
    
    if !from.exists() || !from.join(WINE_EXECUTABLE_PATH).exists() {
        return Err(anyhow!("Path `{}` does not exist or is not a supported Game Porting Toolkit.app", &from.display()));
    }

    let to = (crate::settings::app_support_dir()?).join(GPTK_APP_FILE_NAME);

    if to.exists() {
        std::fs::remove_dir_all(&to).with_context(|| format!("Unable to remove existing bundle `{}`", &to.display()))?;
    }

    let mut process = Command::new("ditto")
        .arg(from)
        .arg(to)
        .spawn()?;

    let status = process.wait()?;

    if status.success() {
        println!("Game Porting Toolkit installed");
    } else {
        println!("Unable to install Game Porting Toolkit: {status}");
    }
    Ok(())
}
