//! `texforge init` command implementation.

use std::path::Path;

use anyhow::Result;

/// Initialize a texforge project in the current directory.
pub fn execute() -> Result<()> {
    let root = std::env::current_dir()?;

    if root.join("project.toml").exists() {
        anyhow::bail!("project.toml already exists in this directory");
    }

    // Detect entry point: file with \documentclass
    let entry = detect_entry(&root).unwrap_or_else(|| "main.tex".to_string());

    // Detect bibliography: first .bib file found
    let bib = detect_bib(&root);

    let bib_line = match &bib {
        Some(b) => format!("bibliografia = \"{}\"", b),
        None => "# bibliografia = \"refs.bib\"".to_string(),
    };

    let project_toml = format!(
        r#"[documento]
titulo = "{}"
autor = "Author"
template = "general"

[compilacion]
entry = "{}"
{}
"#,
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("documento"),
        entry,
        bib_line,
    );

    std::fs::write(root.join("project.toml"), &project_toml)?;

    println!("✅ project.toml generated");
    println!("   entry: {}", entry);
    if let Some(b) = bib {
        println!("   bibliography: {}", b);
    }
    println!();
    println!("Edit project.toml to set titulo and autor, then run:");
    println!("  texforge build");

    Ok(())
}

/// Find the .tex file that contains \documentclass.
fn detect_entry(root: &Path) -> Option<String> {
    for entry in walkdir::WalkDir::new(root)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("tex") {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            if content.contains("\\documentclass") {
                return path
                    .strip_prefix(root)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Find the first .bib file in the project.
fn detect_bib(root: &Path) -> Option<String> {
    for entry in walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("bib") {
            return path
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().to_string());
        }
    }
    None
}
