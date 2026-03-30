//! `texforge fmt` command implementation.

use anyhow::Result;

use crate::domain::project::Project;
use crate::formatter;
use crate::utils;

/// Format .tex files.
pub fn execute(check: bool) -> Result<()> {
    let project = Project::load()?;
    let files = utils::find_tex_files(&project.root)?;

    if files.is_empty() {
        println!("No .tex files found");
        return Ok(());
    }

    let mut unformatted = 0;

    for file in &files {
        let content = std::fs::read_to_string(file)?;
        let formatted = formatter::format(&content);

        if content != formatted {
            let rel = file.strip_prefix(&project.root).unwrap_or(file).display();
            if check {
                println!("  ✗ {}", rel);
                unformatted += 1;
            } else {
                std::fs::write(file, &formatted)?;
                println!("  formatted {}", rel);
            }
        }
    }

    if check && unformatted > 0 {
        anyhow::bail!(
            "{} file(s) need formatting — run 'texforge fmt'",
            unformatted
        );
    } else if !check {
        println!("✅ {} file(s) checked", files.len());
    } else {
        println!("✅ All files formatted correctly");
    }

    Ok(())
}
