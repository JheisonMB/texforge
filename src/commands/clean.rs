//! `texforge clean` command implementation.

use anyhow::Result;

use crate::domain::project::Project;

/// Remove the build/ directory.
pub fn execute() -> Result<()> {
    let project = Project::load()?;
    let build_dir = project.root.join("build");

    if !build_dir.exists() {
        println!("Nothing to clean.");
        return Ok(());
    }

    std::fs::remove_dir_all(&build_dir)?;
    println!("✅ build/ removed");
    Ok(())
}
