//! `texforge new` command implementation.

use anyhow::Result;

/// Create a new project from a template
pub fn execute(name: &str, template: Option<&str>) -> Result<()> {
    let template_name = template.unwrap_or("basic");
    
    println!("Creating project '{}' with template '{}'", name, template_name);
    println!("TODO: Implement project creation");
    
    // TODO:
    // 1. Validate template exists
    // 2. Create project directory
    // 3. Copy template structure
    // 4. Generate project.toml
    // 5. Create main.tex entry point
    
    Ok(())
}
