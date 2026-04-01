//! `texforge build` command implementation.

use anyhow::Result;

use crate::compiler;
use crate::diagrams;
use crate::domain::project::Project;

/// Compile project to PDF.
pub fn execute() -> Result<()> {
    let project = Project::load()?;

    println!("Building project: {}", project.config.documento.titulo);

    std::fs::create_dir_all(project.root.join("build"))?;

    // Pre-process embedded diagrams — works on copies in build/, originals untouched
    let build_entry = diagrams::process(&project.root, &project.config.compilacion.entry)?;

    // Compile from build/ — all assets are mirrored there, diagrams use relative paths
    let build_dir = project.root.join("build");
    let entry_filename = std::path::Path::new(&project.config.compilacion.entry)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(project.config.compilacion.entry.clone());
    compiler::compile(&build_dir, &entry_filename)?;

    let pdf_name = std::path::Path::new(&project.config.compilacion.entry).with_extension("pdf");
    println!("✅ build/{}", pdf_name.display());

    Ok(())
}
