//! `texforge build` command implementation.

use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{RecursiveMode, Watcher};

use crate::compiler;
use crate::diagrams;
use crate::domain::project::Project;

/// Compile project to PDF.
pub fn execute() -> Result<()> {
    let project = Project::load()?;
    println!("Building project: {}", project.config.documento.titulo);
    std::fs::create_dir_all(project.root.join("build"))?;
    diagrams::process(&project.root, &project.config.compilacion.entry)?;
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

/// Watch for .tex file changes and rebuild with debounce.
pub fn watch(delay_secs: u64) -> Result<()> {
    let project = Project::load()?;
    let debounce = Duration::from_secs(delay_secs);

    println!(
        "Watching project: {} ({}s debounce — Ctrl+C to stop)",
        project.config.documento.titulo, delay_secs
    );

    // Initial build
    run_build(&project);

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    watcher.watch(&project.root, RecursiveMode::Recursive)?;

    let build_dir = project.root.join("build");
    let mut pending = false;
    let mut last_event = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                // Only react to .tex file changes, ignore build/
                let relevant = event.paths.iter().any(|p| {
                    !p.starts_with(&build_dir)
                        && p.extension().and_then(|e| e.to_str()) == Some("tex")
                });
                if relevant {
                    pending = true;
                    last_event = std::time::Instant::now();
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        if pending && last_event.elapsed() >= debounce {
            pending = false;
            println!("\n--- rebuilding ---");
            run_build(&project);
        }
    }

    Ok(())
}

fn run_build(project: &Project) {
    let _ = std::fs::create_dir_all(project.root.join("build"));
    if let Err(e) = diagrams::process(&project.root, &project.config.compilacion.entry) {
        eprintln!("Error: {}", e);
        return;
    }
    let build_dir = project.root.join("build");
    let entry_filename = std::path::Path::new(&project.config.compilacion.entry)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(project.config.compilacion.entry.clone());
    match compiler::compile(&build_dir, &entry_filename) {
        Ok(()) => {
            let pdf = std::path::Path::new(&project.config.compilacion.entry).with_extension("pdf");
            println!("✅ build/{}", pdf.display());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
