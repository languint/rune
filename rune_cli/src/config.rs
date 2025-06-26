use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use toml::from_str;

use crate::errors::CliError;

pub fn get_config_file_path(current_directory: &PathBuf) -> PathBuf {
    current_directory.join("Rune.toml")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub title: String,
    pub version: String,
    pub build: BuildConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildConfig {
    pub source_dir: Option<String>,
    pub target_dir: Option<String>,
}

pub fn get_config(current_directory: &PathBuf) -> Result<Config, CliError> {
    let config_path = get_config_file_path(current_directory);

    let config_str = fs::read_to_string(config_path).map_err(|err| {
        CliError::IOError(format!("Failed to read config file (Rune.toml) `{}`", err))
    })?;

    let config: Config =
        from_str(&config_str).map_err(|err| CliError::InvalidConfig(err.to_string()))?;

    Ok(config)
}

pub fn find_target_files(dir: PathBuf, extension: &str) -> Vec<PathBuf> {
    if dir.is_dir() {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir)
            .map_err(|_| CliError::IOError("Failed to read directory".to_string()))
            .unwrap()
        {
            let entry = entry
                .map_err(|_| CliError::IOError("Failed to read directory entry".to_string()))
                .unwrap();

            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
                files.push(path);
            } else if path.is_dir() {
                files.extend(find_target_files(path, extension));
            }
        }
        files
    } else {
        Vec::new()
    }
}

fn get_something(key: usize) -> Option<i32> {
    let values = vec![1, 2, 3, 4, 5, 6, 7];

    values.get(key).map(|v| *v)
}

fn main() {
    if let Some(value) = get_something(3) {
        println!("Something: {}", value)
    } else {
        println!("Something not found")
    };
}
