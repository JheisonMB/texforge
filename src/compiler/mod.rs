//! LaTeX compilation engine — wraps Tectonic.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Compile a LaTeX project to PDF using Tectonic.
pub fn compile(root: &Path, entry: &str) -> Result<()> {
    let tectonic = find_tectonic()?;
    let entry_path = root.join(entry);

    let output = Command::new(&tectonic)
        .arg(&entry_path)
        .arg("--outdir")
        .arg(root.join("build"))
        .arg("--keep-logs")
        .current_dir(root)
        .output()
        .with_context(|| format!("Failed to run tectonic at {}", tectonic.display()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw = format!("{}{}", stdout, stderr);

    let errors = parse_errors(&raw);
    if errors.is_empty() {
        anyhow::bail!("Compilation failed:\n{}", raw.trim());
    }

    let mut msg = String::from("Compilation failed:\n\n");
    for e in &errors {
        msg.push_str(&format!(
            "ERROR [{}:{}]\n  {}\n\n",
            e.file, e.line, e.message
        ));
    }
    anyhow::bail!("{}", msg.trim());
}

struct CompileError {
    file: String,
    line: usize,
    message: String,
}

/// Parse tectonic/TeX error output into structured errors.
fn parse_errors(raw: &str) -> Vec<CompileError> {
    let mut errors = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("error:") {
            let rest = rest.trim();
            if let Some((loc, msg)) = rest.split_once(": ") {
                if let Some((file, line_str)) = loc.rsplit_once(':') {
                    if let Ok(line_num) = line_str.parse::<usize>() {
                        errors.push(CompileError {
                            file: file.trim().to_string(),
                            line: line_num,
                            message: msg.trim().to_string(),
                        });
                        continue;
                    }
                }
            }
            errors.push(CompileError {
                file: String::new(),
                line: 0,
                message: rest.to_string(),
            });
        }

        if let Some(msg) = trimmed.strip_prefix("! ") {
            errors.push(CompileError {
                file: String::new(),
                line: 0,
                message: msg.to_string(),
            });
        }
        if let Some(num_str) = trimmed.strip_prefix("l.") {
            let num_part: String = num_str.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num_part.parse::<usize>() {
                if let Some(last) = errors.last_mut() {
                    last.line = n;
                }
            }
        }
    }

    errors
}

/// Find the tectonic binary in PATH or known locations.
fn find_tectonic() -> Result<std::path::PathBuf> {
    // Check PATH
    if let Ok(output) = Command::new("which").arg("tectonic").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(path.into());
        }
    }

    // Check known locations (including texforge-managed install)
    for candidate in [
        dirs::home_dir().map(|h| h.join(".texforge/bin/tectonic")),
        dirs::home_dir().map(|h| h.join(".cargo/bin/tectonic")),
        Some("/usr/local/bin/tectonic".into()),
        Some("/opt/homebrew/bin/tectonic".into()),
    ]
    .into_iter()
    .flatten()
    {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "Tectonic not found. Install everything with:\n\
         \n  curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh\n\
         \nor install tectonic separately: cargo install tectonic"
    );
}
