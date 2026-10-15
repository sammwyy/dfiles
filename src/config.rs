//! Persistent preferences for File Explorer.

use delib::xdg::XdgPaths;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub explorer: ExplorerConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct ExplorerConfig {
    /// Include dot-prefixed entries when listing a directory.
    pub show_hidden: bool,
    /// Open directory tabs using the grid presentation.
    pub default_grid_view: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            explorer: ExplorerConfig::default(),
        }
    }
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            default_grid_view: false,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        XdgPaths::default().user_config_file("dfiles", "config.toml")
    }

    /// Reads the user's preferences. A malformed or unreadable file never
    /// prevents the file manager from starting; settings can repair it later.
    pub fn load() -> Self {
        let path = Self::path();
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("dfiles: failed to parse {}: {error}", path.display());
                    Self::default()
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                eprintln!("dfiles: failed to read {}: {error}", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("configuration path has no parent"))?;
        fs::create_dir_all(parent)?;
        let contents =
            toml::to_string_pretty(self).map_err(|error| io::Error::other(error.to_string()))?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, contents)?;
        fs::rename(temporary, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_are_stable() {
        let config = Config::default();
        assert!(!config.explorer.show_hidden);
        assert!(!config.explorer.default_grid_view);
        assert!(toml::to_string(&config).unwrap().contains("[explorer]"));
    }
}
