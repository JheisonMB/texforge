//! `texforge build` command implementation.

use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{RecursiveMode, Watcher};

use crate::commands::init::BANNER;
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
    println!("  ◇ build/{}", pdf_name.display());
    Ok(())
}

/// Watch for .tex file changes and rebuild with debounce.
pub fn watch(delay_secs: u64) -> Result<()> {
    let project = Project::load()?;
    let debounce = Duration::from_secs(delay_secs);
    // Ignore new events for this long after a build completes
    let cooldown = Duration::from_secs(2);

    print_watch_header(&project.config.documento.titulo, delay_secs);

    let started = std::time::Instant::now();
    let result = run_build(&project);
    redraw_status(&result, 1, started);

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
    let mut last_build = std::time::Instant::now();
    let mut build_count = 1u32;
    let mut last_result = result;
    let mut last_tick = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                let relevant = event.paths.iter().any(|p| {
                    !p.starts_with(&build_dir)
                        && p.extension().and_then(|e| e.to_str()) == Some("tex")
                });
                if relevant && last_build.elapsed() > cooldown {
                    pending = true;
                    last_event = std::time::Instant::now();
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        // Redraw timer every second even without a build
        if last_tick.elapsed() >= Duration::from_secs(1) {
            last_tick = std::time::Instant::now();
            redraw_status(&last_result, build_count, started);
        }

        if pending && last_event.elapsed() >= debounce {
            pending = false;
            build_count += 1;
            last_result = run_build(&project);
            last_build = std::time::Instant::now();
            redraw_status(&last_result, build_count, started);
        }
    }

    Ok(())
}

fn print_watch_header(title: &str, delay_secs: u64) {
    print!("\x1B[2J\x1B[H");
    println!("{BANNER}");
    println!("  {title} — watching  ({delay_secs}s debounce  Ctrl+C to stop)");
}

fn redraw_status(result: &WatchResult, build_count: u32, started: std::time::Instant) {
    // Move to line 15 (just after header), clear from there down, redraw
    print!("\x1B[15;0H\x1B[J");
    let e = started.elapsed().as_secs();
    let session = format!("{:02}:{:02}:{:02}", e / 3600, (e % 3600) / 60, e % 60);
    println!();
    println!("  session  \x1B[36m{session}\x1B[0m   builds  \x1B[36m{build_count}\x1B[0m");
    println!();
    match result {
        WatchResult::Ok(pdf) => println!("  \x1B[32mbuild/{pdf}  ok\x1B[0m"),
        WatchResult::Err(err) => {
            println!("  \x1B[31merror:\x1B[0m");
            for line in err.lines() {
                println!("    {line}");
            }
        }
    }
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

enum WatchResult {
    Ok(String),
    Err(String),
}

fn run_build(project: &Project) -> WatchResult {
    let _ = std::fs::create_dir_all(project.root.join("build"));
    if let Err(e) = diagrams::process(&project.root, &project.config.compilacion.entry) {
        return WatchResult::Err(e.to_string());
    }
    let build_dir = project.root.join("build");
    let entry_filename = std::path::Path::new(&project.config.compilacion.entry)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(project.config.compilacion.entry.clone());
    match compiler::compile(&build_dir, &entry_filename) {
        Ok(()) => {
            let pdf = std::path::Path::new(&project.config.compilacion.entry).with_extension("pdf");
            WatchResult::Ok(pdf.display().to_string())
        }
        Err(e) => WatchResult::Err(e.to_string()),
    }
}
