//! Static linting rules.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;

/// A linting error with location and suggestion.
#[derive(Debug)]
pub struct LintError {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  {}:{} — {}", self.file, self.line, self.message)?;
        if let Some(ref s) = self.suggestion {
            write!(f, "\n    suggestion: {}", s)?;
        }
        Ok(())
    }
}

/// Run all lint rules on a project directory.
pub fn lint(root: &Path, entry: &str, bib_file: Option<&str>) -> Result<Vec<LintError>> {
    let mut errors = Vec::new();

    let entry_path = root.join(entry);
    if !entry_path.exists() {
        errors.push(LintError {
            file: entry.to_string(),
            line: 0,
            message: "Entry point file does not exist".into(),
            suggestion: Some(format!("Create {}", entry)),
        });
        return Ok(errors);
    }

    // Collect all .tex files reachable from entry
    let mut tex_files = Vec::new();
    collect_tex_files(root, entry, &mut tex_files);

    // Parse .bib keys if bibliography exists
    let bib_keys = match bib_file {
        Some(bib) => parse_bib_keys(&root.join(bib)),
        None => HashSet::new(),
    };

    // Collect all labels defined across files
    let mut all_labels = HashSet::new();
    for file in &tex_files {
        let rel = file.strip_prefix(root).unwrap_or(file);
        let content = std::fs::read_to_string(file)?;
        for (i, line) in content.lines().enumerate() {
            for label in extract_commands(line, "label") {
                all_labels.insert(label.to_string());
            }
            let _ = (i, rel); // used below
        }
    }

    // Run checks on each file
    for file in &tex_files {
        let rel = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(file)?;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            // Check \input{} files exist
            for arg in extract_commands(line, "input") {
                let input_path = resolve_tex_path(root, arg);
                if !input_path.exists() {
                    errors.push(LintError {
                        file: rel.clone(),
                        line: line_num,
                        message: format!("\\input{{{}}} — file not found", arg),
                        suggestion: Some(format!("Create {}", input_path.display())),
                    });
                }
            }

            // Check \includegraphics{} files exist
            for arg in extract_commands(line, "includegraphics") {
                let img_path = root.join(arg);
                if !img_path.exists() {
                    errors.push(LintError {
                        file: rel.clone(),
                        line: line_num,
                        message: format!("\\includegraphics{{{}}} — file not found", arg),
                        suggestion: None,
                    });
                }
            }

            // Check \cite{} keys exist in .bib
            if bib_file.is_some() {
                for arg in extract_commands(line, "cite") {
                    for key in arg.split(',') {
                        let key = key.trim();
                        if !key.is_empty() && !bib_keys.contains(key) {
                            errors.push(LintError {
                                file: rel.clone(),
                                line: line_num,
                                message: format!("\\cite{{{}}} — key not found in .bib", key),
                                suggestion: None,
                            });
                        }
                    }
                }
            }

            // Check \ref{} has matching \label{}
            for arg in extract_commands(line, "ref") {
                if !all_labels.contains(arg) {
                    errors.push(LintError {
                        file: rel.clone(),
                        line: line_num,
                        message: format!("\\ref{{{}}} — no matching \\label found", arg),
                        suggestion: None,
                    });
                }
            }
        }
    }

    Ok(errors)
}

/// Extract arguments from `\command{arg}` and `\command[opts]{arg}` occurrences in a line.
fn extract_commands<'a>(line: &'a str, cmd: &str) -> Vec<&'a str> {
    let mut results = Vec::new();
    let pattern = format!("\\{}", cmd);
    let mut search = line;

    while let Some(pos) = search.find(&pattern) {
        let after = &search[pos + pattern.len()..];
        // Skip optional args [...]
        let after = if after.starts_with('[') {
            match after.find(']') {
                Some(end) => &after[end + 1..],
                None => break,
            }
        } else {
            after
        };
        if after.starts_with('{') {
            if let Some(end) = after.find('}') {
                results.push(&after[1..end]);
                search = &after[end + 1..];
                continue;
            }
        }
        search = after;
    }

    results
}

/// Resolve a tex input path, adding .tex extension if missing.
fn resolve_tex_path(root: &Path, input: &str) -> std::path::PathBuf {
    let p = root.join(input);
    if p.extension().is_some() {
        p
    } else {
        p.with_extension("tex")
    }
}

/// Recursively collect .tex files referenced by \input{}.
fn collect_tex_files(root: &Path, entry: &str, files: &mut Vec<std::path::PathBuf>) {
    let path = resolve_tex_path(root, entry);
    if !path.exists() || files.contains(&path) {
        return;
    }
    files.push(path.clone());

    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            for input in extract_commands(line, "input") {
                collect_tex_files(root, input, files);
            }
        }
    }
}

/// Parse @type{key, ...} entries from a .bib file.
fn parse_bib_keys(path: &Path) -> HashSet<String> {
    let mut keys = HashSet::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return keys;
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') && !trimmed.starts_with("@comment") {
            if let Some(start) = trimmed.find('{') {
                if let Some(end) = trimmed[start..].find(',') {
                    let key = trimmed[start + 1..start + end].trim();
                    if !key.is_empty() {
                        keys.insert(key.to_string());
                    }
                }
            }
        }
    }
    keys
}
