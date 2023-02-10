use anyhow::{anyhow, Result};
use std::{env::current_exe, path::Path};

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

impl Config {
    #[rustfmt::skip]
    pub fn load() -> Result<Self> {
        let selfd = Self::default();
        // hacky way to make read() available w/o an instance
        // why selfd? because both self and Self are unavailable

        let exe = current_exe()?;
        let current_directory = exe
            .parent()
            .ok_or(anyhow!("Failed to get the parent directory of the executable; *This should never happen.*"))?
            .canonicalize()?;


        Ok(match std::fs::read_to_string(current_directory.join("config.yml")) {
            Ok(config) => {
                let config: Config = serde_yaml::from_str(&config).unwrap_or_else(|err| {
                    crate::terminal::error(format!("Failed to parse the config file, reason: {}", err.to_string()));
                    selfd
                });

                config
            }
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        crate::terminal::info("No config file found, creating a new one.");
                        selfd.save()?;
                    }
                    _ => {
                        crate::terminal::error(format!("Failed to read the config file, reason: {}", err.to_string()));
                    }

                }
                selfd
            }
            })
    }

    pub fn save(&self) -> Result<()> {
        let exe = current_exe()?;
        let current_directory = exe
            .parent()
            .ok_or(anyhow!(
                "Failed to get the parent directory of the executable; *This should never happen.*"
            ))?
            .canonicalize()?;
        std::fs::write(
            current_directory.join("config.yml"),
            serde_yaml::to_string(self).unwrap(),
        )?;
        Ok(())
    }
}
