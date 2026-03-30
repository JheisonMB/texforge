//! `texforge check` command implementation.

use anyhow::Result;

use crate::domain::project::Project;

/// Lint project without compiling
pub fn execute() -> Result<()> {
    let project = Project::load()?;

    println!("Checking project: {}", project.config.documento.titulo);
    println!("TODO: Implement linter");

    // TODO:
    // 1. Check \cite{} references exist in .bib
    // 2. Check \includegraphics files exist
    // 3. Check \input files exist
    // 4. Check \label{} / \ref{} consistency
    // 5. Validate project.toml variables
    // 6. Check diagram blocks are well-formed

    Ok(())
}
