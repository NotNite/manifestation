use anyhow::Context;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ManifestationConfig {
    pub godot_path: Option<PathBuf>,
    pub gdweave_path: Option<PathBuf>,
}

fn get_default_config_dir() -> anyhow::Result<PathBuf> {
    Ok(ProjectDirs::from("com", "notnite", "manifestation")
        .context("Failed to setup project directory")?
        .config_dir()
        .to_path_buf())
}

pub fn get_config_dir() -> anyhow::Result<PathBuf> {
    let dir = std::env::var("MANIFESTATION_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or(get_default_config_dir().context("Failed to find project directory")?);

    if !dir.exists() {
        std::fs::create_dir_all(&dir).context("Failed to create project directory")?;
    }

    Ok(dir)
}

pub fn get_config() -> anyhow::Result<ManifestationConfig> {
    let path = get_config_dir()?.join("config.toml");
    let str = std::fs::read_to_string(&path).context("Failed to read config file")?;
    let config = toml::from_str(&str).context("Failed to parse config file")?;
    Ok(config)
}

pub fn set_config(config: ManifestationConfig) -> anyhow::Result<()> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    let config_str = toml::to_string(&config).context("Failed to serialize config")?;
    std::fs::write(&config_path, config_str).context("Failed to write config file")?;
    Ok(())
}

pub fn config() -> anyhow::Result<()> {
    if get_config().is_ok() {
        let rerun = inquire::Confirm::new("You already set up your config file - redo it?")
            .with_default(false)
            .prompt()?;
        if !rerun {
            return Ok(());
        }
    }

    let godot_path = inquire::Text::new("Path to Godot editor executable")
        .with_validator(|path: &str| {
            if !PathBuf::from(path).exists() {
                Ok(inquire::validator::Validation::Invalid(
                    inquire::validator::ErrorMessage::Default,
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()?;

    let gdweave_path = inquire::Text::new("Path to GDWeave directory")
        .with_validator(|path: &str| {
            if !PathBuf::from(path).exists() {
                Ok(inquire::validator::Validation::Invalid(
                    inquire::validator::ErrorMessage::Default,
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()?;

    let config = ManifestationConfig {
        godot_path: Some(PathBuf::from(godot_path)),
        gdweave_path: Some(PathBuf::from(gdweave_path)),
    };

    set_config(config)?;

    Ok(())
}
