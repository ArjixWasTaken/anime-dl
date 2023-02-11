use anyhow::{anyhow, bail, Result};
use std::{
    env::current_exe,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

macro_rules! decl_default {
    ($name:ident, $value:expr) => {
        pub(crate) fn $name() -> String {
            $value.to_string()
        }
    };
}

#[rustfmt::skip]
decl_default!(default_file_format, "{anime_title}/{episode_title}_{episode_number}.{file_format}");
decl_default!(default_provider, crate::cli::PROVIDERS.first().unwrap());
decl_default!(default_path, ".");

#[derive(Serialize, Deserialize)]
pub struct DlCmdConfig {
    #[serde(default = "default_path")]
    download_directory: String,
    #[serde(default = "default_file_format")]
    file_format: String,
    #[serde(default = "default_provider")]
    default_provider: String,
}

#[derive(Serialize, Deserialize)]
pub struct CommandsConfig {
    pub dl: DlCmdConfig,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub check_for_updates: bool,
    pub verbosity: u64,
    pub commands: CommandsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            verbosity: 0,
            commands: CommandsConfig {
                dl: DlCmdConfig {
                    download_directory: default_path(),
                    file_format: default_file_format(),
                    default_provider: default_provider(),
                },
            },
        }
    }
}

#[rustfmt::skip]
fn find_config_file(filename: &str) -> Result<PathBuf> {
    if let Ok(exe) = current_exe() {
        let exe_dir = exe.parent().ok_or(anyhow!("Failed to get the parent directory of the executable; *This should never happen.*"))?;
        let config_file = exe_dir.join(filename);

        if config_file.exists() {
            return Ok(config_file);
        }
    }

    let config_folder = if cfg!(target_os = "linux") {
        std::env::var("XDG_CONFIG_HOME")
            .map(|x| Path::new(&x).to_path_buf())
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| {
                    panic!("Failed to get the home directory, please set the XDG_CONFIG_HOME environment variable.")
                });
                Path::new(&home).join(".config")
            })
    } else if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
            .map(|p| Path::new(&p).join(".config"))
            .unwrap_or_else(|_| {
                Path::new(&std::env::var("APPDATA").unwrap()).to_path_buf()
            })
    } else {
        bail!("Unsupported operating system. {:?}", std::env::consts::OS);
    }.join("anime-dl");

    Ok(config_folder.join(filename))
}

impl Config {
    #[rustfmt::skip]
    pub fn load() -> Result<Self> {
        let self_ = Self::default();

        let config_file = find_config_file("config.yml")?;

        Ok(match std::fs::read_to_string(config_file) {
            Ok(config) => {
                let config: Config = serde_yaml::from_str(&config).unwrap_or_else(|err| {
                    crate::terminal::error(format!("Failed to parse the config file, reason: {}", err.to_string()));
                    self_
                });

                config
            }
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        crate::terminal::info("No config file found, creating a new one.");
                        self_.save()?;
                    }
                    _ => {
                        crate::terminal::error(format!("Failed to read the config file, reason: {}", err.to_string()));
                    }

                }
                self_
            }
            })
    }

    pub fn save(&self) -> Result<()> {
        let config_file = find_config_file("config.yml")?;
        if !config_file.parent().unwrap().exists() {
            std::fs::create_dir_all(config_file.parent().unwrap())?;
        }

        std::fs::write(config_file, serde_yaml::to_string(self).unwrap())?;
        Ok(())
    }
}
