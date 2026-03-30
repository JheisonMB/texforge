//! Project configuration and metadata.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Project configuration from project.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub documento: DocumentoConfig,
    pub compilacion: CompilacionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentoConfig {
    pub titulo: String,
    pub autor: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilacionConfig {
    pub entry: String,
    #[serde(default)]
    pub bibliografia: Option<String>,
}

/// Represents a `TexForge` project
#[derive(Debug)]
pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    /// Load project from current directory
    pub fn load() -> anyhow::Result<Self> {
        let root = std::env::current_dir()?;
        let config_path = root.join("project.toml");

        if !config_path.exists() {
            anyhow::bail!("No project.toml found in current directory");
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;

        Ok(Self { root, config })
    }

    /// Get the entry point file path
    pub fn entry_path(&self) -> PathBuf {
        self.root.join(&self.config.compilacion.entry)
    }

    /// Get the bibliography file path if configured
    pub fn bib_path(&self) -> Option<PathBuf> {
        self.config
            .compilacion
            .bibliografia
            .as_ref()
            .map(|bib| self.root.join(bib))
    }
}
