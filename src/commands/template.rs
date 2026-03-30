//! `texforge template` command implementation.

use anyhow::Result;

use crate::templates;

/// List available templates.
pub fn list() -> Result<()> {
    let cached = templates::list_cached()?;
    if cached.is_empty() {
        println!("No templates installed locally.");
        println!("The 'general' template is always available (built-in).");
    } else {
        println!("Installed templates:");
        for name in &cached {
            println!("  - {}", name);
        }
    }
    Ok(())
}

/// Add a template from the registry.
pub fn add(name: &str) -> Result<()> {
    println!("Downloading template '{}'...", name);
    templates::download(name)?;
    println!("✅ Template '{}' installed", name);
    Ok(())
}

/// Remove a template from local cache.
pub fn remove(name: &str) -> Result<()> {
    let path = templates::remove_cached(name)?;
    println!("✅ Removed template '{}' ({})", name, path.display());
    Ok(())
}

/// Validate template compatibility.
pub fn validate(name: &str) -> Result<()> {
    let resolved = templates::resolve(name)?;
    if resolved.files.contains_key("template.toml") {
        println!("✅ Template '{}' is valid", name);
    } else {
        anyhow::bail!("Template '{}' is missing template.toml", name);
    }
    Ok(())
}
