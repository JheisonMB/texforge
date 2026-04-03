//! `texforge init` command implementation — interactive wizard.

use std::path::Path;

use anyhow::Result;
use inquire::{Select, Text};

use crate::commands::new as new_cmd;
use crate::templates;

pub(crate) const BANNER: &str = r#"
 ███████████          █████ █████ ███████████                                     
░█░░░███░░░█         ░░███ ░░███ ░░███░░░░░░█                                     
░   ░███  ░   ██████  ░░███ ███   ░███   █ ░   ██████  ████████   ███████  ██████ 
    ░███     ███░░███  ░░█████    ░███████    ███░░███░░███░░███ ███░░███ ███░░███
    ░███    ░███████    ███░███   ░███░░░█   ░███ ░███ ░███ ░░░ ░███ ░███░███████ 
    ░███    ░███░░░    ███ ░░███  ░███  ░    ░███ ░███ ░███     ░███ ░███░███░░░  
    █████   ░░██████  █████ █████ █████      ░░██████  █████    ░░███████░░██████ 
   ░░░░░     ░░░░░░  ░░░░░ ░░░░░ ░░░░░        ░░░░░░  ░░░░░      ░░░░░███ ░░░░░░  
                                                                 ███ ░███         
                                                                ░░██████          
                                                                 ░░░░░░           
"#;

/// Interactive wizard: migrate existing project or scaffold a new one.
pub fn execute() -> Result<()> {
    println!("{BANNER}");

    let root = std::env::current_dir()?;

    if root.join("project.toml").exists() {
        anyhow::bail!("project.toml already exists — nothing to do");
    }

    let has_tex = detect_entry(&root).is_some();

    if has_tex {
        migrate(&root)
    } else {
        create_new()
    }
}

fn migrate(root: &Path) -> Result<()> {
    let entry = detect_entry(root).unwrap_or_else(|| "main.tex".to_string());
    let bib = detect_bib(root);

    let titulo = Text::new("Document title")
        .with_default(
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("documento"),
        )
        .prompt()?;

    let autor = Text::new("Author").with_default("Author").prompt()?;

    let bib_line = match &bib {
        Some(b) => format!("bibliografia = \"{}\"", b),
        None => "# bibliografia = \"refs.bib\"".to_string(),
    };

    let project_toml = format!(
        "[documento]\ntitulo = \"{titulo}\"\nautor = \"{autor}\"\ntemplate = \"general\"\n\n[compilacion]\nentry = \"{entry}\"\n{bib_line}\n"
    );

    std::fs::write(root.join("project.toml"), &project_toml)?;

    println!("\n  ◇ project.toml generated  ✓");
    println!("  ◇ entry: {entry}");
    if let Some(b) = &bib {
        println!("  ◇ bibliography: {b}");
    }
    println!("\n  Run: texforge build\n");
    Ok(())
}

fn create_new() -> Result<()> {
    let name = Text::new("Project name")
        .with_help_message("e.g. mi-tesis  (no spaces)")
        .prompt()?;
    new_cmd::validate_project_name(&name)?;

    let template = select_template()?;

    println!();
    new_cmd::execute(&name, Some(&template))?;
    Ok(())
}

fn select_template() -> Result<String> {
    let mut options: Vec<String> = vec!["general  (built-in, works offline)".to_string()];

    // Try to fetch remote templates; silently fall back if offline
    if let Ok(remote) = templates::list_remote() {
        for t in remote {
            if t != "general" {
                options.push(t);
            }
        }
    }

    // Also add locally cached ones not already listed
    if let Ok(cached) = templates::list_cached() {
        for t in cached {
            if !options.iter().any(|o| o.starts_with(&t)) {
                options.push(t);
            }
        }
    }

    let selected = Select::new("Template", options)
        .with_help_message("↑↓ move  enter confirm")
        .prompt()?;

    // Extract just the template name (before any spaces)
    Ok(selected
        .split_whitespace()
        .next()
        .unwrap_or("general")
        .to_string())
}

/// Find the .tex file that contains \documentclass.
fn detect_entry(root: &Path) -> Option<String> {
    find_file_by(root, 2, |path| {
        path.extension().and_then(|e| e.to_str()) == Some("tex")
            && std::fs::read_to_string(path)
                .map(|c| c.contains("\\documentclass"))
                .unwrap_or(false)
    })
}

/// Find the first .bib file in the project.
fn detect_bib(root: &Path) -> Option<String> {
    find_file_by(root, 3, |path| {
        path.extension().and_then(|e| e.to_str()) == Some("bib")
    })
}

fn find_file_by(
    root: &Path,
    max_depth: usize,
    predicate: impl Fn(&Path) -> bool,
) -> Option<String> {
    walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_file() && predicate(e.path()))
        .and_then(|e| {
            e.path()
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
}
