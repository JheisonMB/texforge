//! `texforge new` command implementation.

use std::path::{Component, Path};

use anyhow::{Context, Result};

use crate::templates;

/// Create a new project from a template.
pub fn execute(name: &str, template: Option<&str>) -> Result<()> {
    validate_project_name(name)?;

    let template_name = template.unwrap_or("general");
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!(
        "Creating project '{}' with template '{}'...",
        name, template_name
    );

    let resolved = templates::resolve(template_name)?;

    // Create project directory and write all template files
    for (rel_path, content) in &resolved.files {
        // Skip template.toml — it's metadata, not a project file
        if rel_path == "template.toml" {
            continue;
        }
        let dest = project_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, content)
            .with_context(|| format!("Failed to write {}", dest.display()))?;
    }

    // Generate project.toml
    let project_toml = format!(
        r#"[documento]
titulo = "{name}"
autor = "Author"
template = "{template_name}"

[compilacion]
entry = "main.tex"
bibliografia = "bib/references.bib"
"#
    );
    std::fs::write(project_dir.join("project.toml"), project_toml)?;

    // Ensure assets/images directory exists
    std::fs::create_dir_all(project_dir.join("assets/images"))?;

    println!("✅ Project '{}' created successfully", name);
    println!();
    println!("  cd {}", name);
    println!("  texforge build");

    Ok(())
}

/// Validate project name: no empty, no path traversal, no special chars.
fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }

    // Reject path traversal
    let path = Path::new(name);
    for component in path.components() {
        match component {
            Component::ParentDir => {
                anyhow::bail!("Project name cannot contain '..' (path traversal)");
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("Project name cannot be an absolute path");
            }
            _ => {}
        }
    }

    // Reject names with slashes (implicit subdirectories)
    if name.contains('/') || name.contains('\\') {
        anyhow::bail!("Project name cannot contain path separators");
    }

    // Reject problematic characters
    let invalid_chars = ['@', '#', '$', '!', '&', '|', ';', '`', '"', '\'', '*', '?'];
    if let Some(c) = name.chars().find(|c| invalid_chars.contains(c)) {
        anyhow::bail!("Project name contains invalid character: '{}'", c);
    }

    // Reject names that are only whitespace
    if name.trim().is_empty() {
        anyhow::bail!("Project name cannot be only whitespace");
    }

    Ok(())
}
