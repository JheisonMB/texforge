//! `texforge new` command implementation.

use std::path::Path;

use anyhow::{Context, Result};

use crate::templates;

/// Create a new project from a template.
pub fn execute(name: &str, template: Option<&str>) -> Result<()> {
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
