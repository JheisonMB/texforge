//! `texforge build` command implementation.

use anyhow::Result;

use crate::domain::project::Project;

/// Compile project to PDF
pub fn execute() -> Result<()> {
    let project = Project::load()?;
    
    println!("Building project: {}", project.config.documento.titulo);
    println!("Entry point: {}", project.entry_path().display());
    println!("TODO: Implement compilation");
    
    // TODO:
    // 1. Load template
    // 2. Assemble document (preamble + body)
    // 3. Render embedded diagrams (Mermaid, Graphviz)
    // 4. Compile with internal engine
    // 5. Report clean errors on failure
    
    Ok(())
}
