//! Configuration commands: simple key access and interactive wizard
//!
//! Usage:
//!   texforge config                      # Interactive wizard
//!   texforge config list                 # List all settings
//!   texforge config name                 # Show value
//!   texforge config name "Jheison"       # Set value

use crate::config;
use anyhow::Result;
use inquire::{Select, Text};

const AVAILABLE_LANGUAGES: &[&str] = &[
    "english",
    "spanish",
    "french",
    "german",
    "portuguese",
    "italian",
    "dutch",
    "russian",
    "chinese",
    "japanese",
];

const BANNER: &str = r#"
 ███████████          █████ █████ ███████████                                     
░█░░░███░░░█         ░░███ ░░███ ░░███░░░░░░█                                     
░   ░███  ░   ██████  ░░███ ███   ░███   █ ░   ██████  ████████   ███████  ██████ 
    ░███     ███░░███  ░░█████    ░███████    ███░░███░░███░░███ ███░░███ ███░░███
    ░███    ░███████    ███░███   ░███░░░█   ░███ ░███ ░███ ░░░ ░███ ░███░███████ 
    ░███    ░███░░░    ███ ░░███  ░███  ░    ░███ ░███ ░███     ░███ ░███░███░░░  
    █████   ░░██████  █████ █████ █████      ░░██████  █████    ░░███████░░██████ 
   ░░░░░     ░░░░░░  ░░░░░ ░░░░░ ░░░░░        ░░░░░░  ░░░░░      ░░░░░███ ░░░░░░  
                                                                 ███ ░███         
                                                                ░░██████          
                                                                 ░░░░░░           
"#;

/// Get a config value by key (e.g., "name", "email", "institution", "language")
pub fn get(key: &str) -> Result<()> {
    let config = config::load()?;
    
    match key {
        "name" => {
            if let Some(name) = &config.user.name {
                println!("{}", name);
            } else {
                println!("(not set)");
            }
        }
        "email" => {
            if let Some(email) = &config.user.email {
                println!("{}", email);
            } else {
                println!("(not set)");
            }
        }
        "institution" => {
            if let Some(inst) = &config.institution.name {
                println!("{}", inst);
            } else {
                println!("(not set)");
            }
        }
        "language" => {
            if let Some(lang) = &config.defaults.language {
                println!("{}", lang);
            } else {
                println!("(not set)");
            }
        }
        _ => {
            anyhow::bail!("Unknown config key: {}. Available: name, email, institution, language", key);
        }
    }
    
    Ok(())
}

/// Set a config value by key
pub fn set(key: &str, value: &str) -> Result<()> {
    let mut config = config::load()?;
    
    match key {
        "name" => {
            config.user.name = Some(value.to_string());
        }
        "email" => {
            config.user.email = Some(value.to_string());
        }
        "institution" => {
            config.institution.name = Some(value.to_string());
        }
        "language" => {
            config.defaults.language = Some(value.to_string());
        }
        _ => {
            anyhow::bail!("Unknown config key: {}. Available: name, email, institution, language", key);
        }
    }
    
    config::save(&config)?;
    println!("✓ Set {} = {}", key, value);
    Ok(())
}

/// List all configuration values
pub fn list() -> Result<()> {
    let config = config::load()?;
    
    println!("Global configuration:\n");
    
    println!("[User]");
    if let Some(name) = &config.user.name {
        println!("  name       = {}", name);
    } else {
        println!("  name       = (not set)");
    }
    if let Some(email) = &config.user.email {
        println!("  email      = {}", email);
    } else {
        println!("  email      = (not set)");
    }
    
    println!();
    println!("[Institution]");
    if let Some(inst) = &config.institution.name {
        println!("  name       = {}", inst);
    } else {
        println!("  name       = (not set)");
    }
    
    println!();
    println!("[Defaults]");
    if let Some(lang) = &config.defaults.language {
        println!("  language   = {}", lang);
    } else {
        println!("  language   = (not set)");
    }
    
    Ok(())
}

/// Interactive configuration wizard - asks for all 4 main fields
pub fn wizard() -> Result<()> {
    println!("{BANNER}");
    println!("Configuration Wizard\n");
    println!("Fill in your details to be used as placeholders in templates:\n");
    
    let config = config::load()?;
    
    let name = Text::new("Name")
        .with_default(config.user.name.as_deref().unwrap_or(""))
        .prompt()?;
    
    let email = Text::new("Email")
        .with_default(config.user.email.as_deref().unwrap_or(""))
        .prompt()?;
    
    let institution = Text::new("Institution")
        .with_default(config.institution.name.as_deref().unwrap_or(""))
        .prompt()?;
    
    let default_lang = config.defaults.language.as_deref().unwrap_or("english");
    let language_options: Vec<&str> = AVAILABLE_LANGUAGES.to_vec();
    
    let selected_language = if AVAILABLE_LANGUAGES.contains(&default_lang) {
        Select::new("Language", language_options)
            .with_help_message("↑↓ move  enter confirm")
            .prompt_skippable()
            .map(|opt| opt.unwrap_or(default_lang))
    } else {
        Select::new("Language", language_options)
            .with_help_message("↑↓ move  enter confirm")
            .prompt()
    };
    
    let language = selected_language?;
    
    // Save all values
    let mut new_config = config::load()?;
    new_config.user.name = Some(name);
    new_config.user.email = Some(email);
    new_config.institution.name = Some(institution);
    new_config.defaults.language = Some(language.to_string());
    
    config::save(&new_config)?;
    
    println!("\n✓ Configuration saved!");
    Ok(())
}
