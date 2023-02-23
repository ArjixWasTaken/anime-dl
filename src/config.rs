use anyhow::{anyhow, bail, Result};
use std::{
    env::current_exe,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use std::any::Any;

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

//#[derive(Serialize, Deserialize, fake_reflect::hello_world,Clone)]
//pub struct DlCmdConfig {
//    #[serde(default = "default_path")]
//    download_directory: String,
//    #[serde(default = "default_file_format")]
//    file_format: String,
////    #[serde(default = "default_provider")]
////    default_provider: String,
//}
//
//#[derive(Serialize, Deserialize,fake_reflect::hello_world,Clone)]
//pub struct CommandsConfig {
//    pub dl: DlCmdConfig,
//}

#[derive(Serialize, Deserialize, fake_reflect::hello_world, Clone)]
pub struct Config {
    pub check_for_updates: bool,
    pub verbosity: u64,
    //pub commands: CommandsConfig,
    #[serde(default = "default_path")]
    pub download_directory: String,
    #[serde(default = "default_file_format")]
    pub file_format: String,
    #[serde(default = "default_provider")]
    pub default_provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            verbosity: 0,
            download_directory: default_path(),
            file_format: default_file_format(),
            default_provider: default_provider()
        }
    }
}

#[rustfmt::skip]
fn find_config_file(filename: &str, check_exe_directory: bool) -> Result<PathBuf> {
    if check_exe_directory {
        if let Ok(exe) = current_exe() {
            let exe_dir = exe.parent().ok_or(anyhow!("Failed to get the parent directory of the executable; *This should never happen.*"))?;
            let config_file = exe_dir.join(filename);

            // If a config file exists in the same directory as the executable, use that.
            if config_file.exists() {
                return Ok(config_file);
            }
        }
    }

    let config_folder = match std::env::consts::OS {
        "windows" => {
            std::env::var("USERPROFILE") // AKA: C:/Users/%username%/
                .map(|p| Path::new(&p).join(".config")) // AKA: C:/Users/%username%/.config
                .unwrap_or_else(|_| {
                    Path::new(&std::env::var("APPDATA").unwrap()).to_path_buf()
                     // AKA: C:/Users/%username%/AppData/Roaming
                     // I can't imagine a scenario where this fallback will be needed, but oh well.
                })
        },
        "linux" | "macos" => {
            std::env::var("XDG_CONFIG_HOME") // AKA: /home/%username%/.config
                .map(|x| Path::new(&x).to_path_buf())
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| {
                        panic!("Failed to get the home directory, please set the XDG_CONFIG_HOME environment variable.")
                    });
                    Path::new(&home).join(".config")
                })
        },
        _ => bail!("Unsupported operating system. {:?}", std::env::consts::OS),
    }.join("anime-dl");

    Ok(config_folder.join(filename))
}

impl Config {
    #[rustfmt::skip]
    pub fn load() -> Result<Self> {
        let self_ = Self::default();

        let config_file = find_config_file("config.yml", true)?;

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
        let mut config_file = find_config_file("config.yml", true)?;

        if !config_file.parent().unwrap().exists() {
            // Lets hope this never fails...
            std::fs::create_dir_all(config_file.parent().unwrap())?;
        }

        match std::fs::write(config_file, serde_yaml::to_string(self).unwrap()) {
            Err(err) => match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    config_file = find_config_file("config.yml", false)?;

                    if !config_file.parent().unwrap().exists() {
                        // Lets hope this never fails...
                        std::fs::create_dir_all(config_file.parent().unwrap())?;
                    }

                    std::fs::write(config_file, serde_yaml::to_string(self).unwrap())?;
                }
                _ => {
                    #[rustfmt::skip]
                    bail!("Failed to write to the config file, reason: {}", err.to_string());
                }
            },
            _ => {}
        }
        Ok(())
    }
}
