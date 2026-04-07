//! Global user configuration system.
//!
//! Stores user preferences in `~/.texforge/config.toml` or `$XDG_CONFIG_HOME/texforge/config.toml`.
//! Supports TOML format with user, institution, defaults, and templates sections.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Global configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub institution: InstitutionConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub templates: TemplatesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstitutionConfig {
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    pub documentclass: Option<String>,
    pub fontsize: Option<String>,
    pub papersize: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplatesConfig {
    pub source: Option<String>,
    pub auto_update: Option<bool>,
    pub watch: Option<bool>,
}

/// Returns the path to the user's texforge config directory.
/// Prefers `$XDG_CONFIG_HOME/texforge`, falls back to `~/.texforge/config.toml`.
#[allow(dead_code)]
fn config_dir() -> Result<PathBuf> {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("texforge");
        return Ok(path);
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(home.join(".texforge"))
}

/// Returns the path to the config.toml file
pub fn config_file_path() -> Result<PathBuf> {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("texforge/config.toml");
        return Ok(path);
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(home.join(".texforge/config.toml"))
}

/// Load config from `~/.texforge/config.toml` or `XDG_CONFIG_HOME/texforge/config.toml`
pub fn load() -> Result<Config> {
    let path = config_file_path()?;

    if !path.exists() {
        // Return empty config if file doesn't exist
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from_str(&content).context("Failed to parse config TOML")
}

/// Save config to `~/.texforge/config.toml` or `XDG_CONFIG_HOME/texforge/config.toml`
pub fn save(config: &Config) -> Result<()> {
    let path = config_file_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("Invalid config path"))?;

    // Create directory if it doesn't exist
    fs::create_dir_all(dir).context("Failed to create config directory")?;

    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    Ok(())
}

/// Get a config value by dotted key (e.g., "user.name", "defaults.fontsize")
pub fn get(key: &str) -> Result<Option<String>> {
    let config = load()?;
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(anyhow!("Invalid config key: {}", key));
    }

    match parts[0] {
        "user" => match parts.get(1) {
            Some(&"name") => Ok(config.user.name),
            Some(&"email") => Ok(config.user.email),
            _ => Err(anyhow!("Unknown user key: {}", key)),
        },
        "institution" => match parts.get(1) {
            Some(&"name") => Ok(config.institution.name),
            Some(&"address") => Ok(config.institution.address),
            _ => Err(anyhow!("Unknown institution key: {}", key)),
        },
        "defaults" => match parts.get(1) {
            Some(&"documentclass") => Ok(config.defaults.documentclass),
            Some(&"fontsize") => Ok(config.defaults.fontsize),
            Some(&"papersize") => Ok(config.defaults.papersize),
            Some(&"language") => Ok(config.defaults.language),
            _ => Err(anyhow!("Unknown defaults key: {}", key)),
        },
        "templates" => match parts.get(1) {
            Some(&"source") => Ok(config.templates.source),
            Some(&"auto_update") => Ok(config.templates.auto_update.map(|b| b.to_string())),
            Some(&"watch") => Ok(config.templates.watch.map(|b| b.to_string())),
            _ => Err(anyhow!("Unknown templates key: {}", key)),
        },
        _ => Err(anyhow!("Unknown config section: {}", parts[0])),
    }
}

/// Set a config value by dotted key
pub fn set(key: &str, value: &str) -> Result<()> {
    let mut config = load()?;
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(anyhow!("Invalid config key: {}", key));
    }

    match parts[0] {
        "user" => match parts.get(1) {
            Some(&"name") => {
                config.user.name = Some(value.to_string());
            }
            Some(&"email") => {
                config.user.email = Some(value.to_string());
            }
            _ => return Err(anyhow!("Unknown user key: {}", key)),
        },
        "institution" => match parts.get(1) {
            Some(&"name") => {
                config.institution.name = Some(value.to_string());
            }
            Some(&"address") => {
                config.institution.address = Some(value.to_string());
            }
            _ => return Err(anyhow!("Unknown institution key: {}", key)),
        },
        "defaults" => match parts.get(1) {
            Some(&"documentclass") => {
                config.defaults.documentclass = Some(value.to_string());
            }
            Some(&"fontsize") => {
                config.defaults.fontsize = Some(value.to_string());
            }
            Some(&"papersize") => {
                config.defaults.papersize = Some(value.to_string());
            }
            Some(&"language") => {
                config.defaults.language = Some(value.to_string());
            }
            _ => return Err(anyhow!("Unknown defaults key: {}", key)),
        },
        "templates" => match parts.get(1) {
            Some(&"source") => {
                config.templates.source = Some(value.to_string());
            }
            Some(&"auto_update") => {
                config.templates.auto_update = Some(value.parse::<bool>()?);
            }
            Some(&"watch") => {
                config.templates.watch = Some(value.parse::<bool>()?);
            }
            _ => return Err(anyhow!("Unknown templates key: {}", key)),
        },
        _ => return Err(anyhow!("Unknown config section: {}", parts[0])),
    }

    save(&config)
}

/// List all configuration values as a flat map
pub fn list_all() -> Result<HashMap<String, String>> {
    let config = load()?;
    let mut map = HashMap::new();

    if let Some(name) = &config.user.name {
        map.insert("user.name".to_string(), name.clone());
    }
    if let Some(email) = &config.user.email {
        map.insert("user.email".to_string(), email.clone());
    }

    if let Some(name) = &config.institution.name {
        map.insert("institution.name".to_string(), name.clone());
    }
    if let Some(address) = &config.institution.address {
        map.insert("institution.address".to_string(), address.clone());
    }

    if let Some(dc) = &config.defaults.documentclass {
        map.insert("defaults.documentclass".to_string(), dc.clone());
    }
    if let Some(fs) = &config.defaults.fontsize {
        map.insert("defaults.fontsize".to_string(), fs.clone());
    }
    if let Some(ps) = &config.defaults.papersize {
        map.insert("defaults.papersize".to_string(), ps.clone());
    }
    if let Some(lang) = &config.defaults.language {
        map.insert("defaults.language".to_string(), lang.clone());
    }

    if let Some(source) = &config.templates.source {
        map.insert("templates.source".to_string(), source.clone());
    }
    if let Some(auto) = config.templates.auto_update {
        map.insert("templates.auto_update".to_string(), auto.to_string());
    }
    if let Some(watch) = config.templates.watch {
        map.insert("templates.watch".to_string(), watch.to_string());
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[user]
name = "Jane Doe"
email = "jane@example.com"

[defaults]
documentclass = "article"
fontsize = "11pt"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.user.name, Some("Jane Doe".to_string()));
        assert_eq!(config.defaults.fontsize, Some("11pt".to_string()));
    }

    #[test]
    fn test_serialize_config() {
        let mut config = Config::default();
        config.user.name = Some("John Doe".to_string());
        config.user.email = Some("john@example.com".to_string());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("John Doe"));
        assert!(toml_str.contains("john@example.com"));
    }
}
