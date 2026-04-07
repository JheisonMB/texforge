//! Configuration commands: get, set, list

use crate::config;
use anyhow::Result;

/// Get a config value
pub fn get(key: &str) -> Result<()> {
    match config::get(key)? {
        Some(value) => println!("{}", value),
        None => println!("(not set)"),
    }
    Ok(())
}

/// Set a config value
pub fn set(key: &str, value: &str) -> Result<()> {
    config::set(key, value)?;
    println!("✓ Set {} = {}", key, value);
    Ok(())
}

/// List all config values
pub fn list() -> Result<()> {
    let values = config::list_all()?;

    if values.is_empty() {
        println!("No configuration set. Use 'texforge config set <key> <value>' to get started.");
        return Ok(());
    }

    println!("Global configuration (~/.texforge/config.toml):\n");

    let mut keys: Vec<_> = values.keys().collect();
    keys.sort();

    let mut current_section = String::new();
    for key in keys {
        let section = key.split('.').next().unwrap_or("");
        if section != current_section {
            if !current_section.is_empty() {
                println!();
            }
            println!("[{}]", section);
            current_section = section.to_string();
        }

        let value = &values[key];
        println!("  {} = {}", key.split('.').nth(1).unwrap_or(key), value);
    }

    Ok(())
}
