use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils::expand_tilde;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
    pub editor: Option<String>,
    pub git_ui: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub editor: String,
    pub git_ui: String,
}

impl ResolvedConfig {
    pub fn default_setting() -> Self {
        Self {
            editor: "nvim".to_string(),
            git_ui: "lazygit".to_string(),
        }
    }

    /// Merge with precedence: CLI > File > Defaults
    pub fn resolve(cli: &crate::cli::Cli, file: FileConfig) -> Self {
        let d = Self::default_setting();
        Self {
            editor: cli.editor.clone().or(file.editor).unwrap_or(d.editor),
            git_ui: cli.git_ui.clone().or(file.git_ui).unwrap_or(d.git_ui),
        }
    }
}

pub fn default_config_path() -> PathBuf {
    expand_tilde("~/.config/projctl/config.toml")
}

pub fn load_config(path: &Path) -> Result<FileConfig> {
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let cfg: FileConfig =
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(cfg)
}
