//! `texforge template` command implementation.

use anyhow::Result;

/// List available templates
pub fn list() -> Result<()> {
    println!("Available templates:");
    println!("TODO: List templates from local cache");
    Ok(())
}

/// Add a template from URL or registry
pub fn add(source: &str) -> Result<()> {
    println!("Adding template from: {}", source);
    println!("TODO: Download and validate template");
    Ok(())
}

/// Remove a template
pub fn remove(name: &str) -> Result<()> {
    println!("Removing template: {}", name);
    println!("TODO: Remove template from local cache");
    Ok(())
}

/// Validate template compatibility
pub fn validate(name: &str) -> Result<()> {
    println!("Validating template: {}", name);
    println!("TODO: Check template compatibility with internal engine");
    Ok(())
}
