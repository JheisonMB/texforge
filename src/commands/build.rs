//! `texforge build` command implementation.

use anyhow::Result;

use crate::compiler;
use crate::domain::project::Project;

/// Compile project to PDF.
pub fn execute() -> Result<()> {
    let project = Project::load()?;

    println!("Building project: {}", project.config.documento.titulo);

    // Ensure build directory exists
    std::fs::create_dir_all(project.root.join("build"))?;

    compiler::compile(&project.root, &project.config.compilacion.entry)?;

    let pdf_name = std::path::Path::new(&project.config.compilacion.entry).with_extension("pdf");
    println!("✅ build/{}", pdf_name.display());

    Ok(())
}
