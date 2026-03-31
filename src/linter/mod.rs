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
    collect_tex_files(root, entry, &mut tex_files, &mut errors);

    // Parse .bib keys if bibliography exists
    let bib_keys = match bib_file {
        Some(bib) => parse_bib_keys(&root.join(bib)),
        None => HashSet::new(),
    };

    // Collect all labels defined across files
    let mut all_labels = HashSet::new();
    for file in &tex_files {
        let content = std::fs::read_to_string(file)?;
        for line in content.lines() {
            let line = strip_comment(line);
            for label in extract_commands(&line, "label") {
                all_labels.insert(label.to_string());
            }
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

        check_references(
            root,
            &rel,
            &content,
            bib_file,
            &bib_keys,
            &all_labels,
            &mut errors,
        );
        check_environments(&rel, &content, &mut errors);
    }

    Ok(errors)
}

/// Check \input, \includegraphics, \cite, \ref references.
fn check_references(
    root: &Path,
    rel: &str,
    content: &str,
    bib_file: Option<&str>,
    bib_keys: &HashSet<String>,
    all_labels: &HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let line = strip_comment(line);

        for arg in extract_commands(&line, "input") {
            let input_path = resolve_tex_path(root, arg);
            if !input_path.exists() {
                errors.push(LintError {
                    file: rel.to_string(),
                    line: line_num,
                    message: format!("\\input{{{}}} — file not found", arg),
                    suggestion: Some(format!("Create {}", input_path.display())),
                });
            }
        }

        for arg in extract_commands(&line, "includegraphics") {
            let img_path = root.join(arg);
            if !img_path.exists() {
                errors.push(LintError {
                    file: rel.to_string(),
                    line: line_num,
                    message: format!("\\includegraphics{{{}}} — file not found", arg),
                    suggestion: None,
                });
            }
        }

        if bib_file.is_some() {
            for arg in extract_commands(&line, "cite") {
                for key in arg.split(',') {
                    let key = key.trim();
                    if !key.is_empty() && !bib_keys.contains(key) {
                        errors.push(LintError {
                            file: rel.to_string(),
                            line: line_num,
                            message: format!("\\cite{{{}}} — key not found in .bib", key),
                            suggestion: None,
                        });
                    }
                }
            }
        }

        for arg in extract_commands(&line, "ref") {
            if !all_labels.contains(arg) {
                errors.push(LintError {
                    file: rel.to_string(),
                    line: line_num,
                    message: format!("\\ref{{{}}} — no matching \\label found", arg),
                    suggestion: None,
                });
            }
        }
    }
}

/// Check for unclosed \begin{env} environments.
fn check_environments(rel: &str, content: &str, errors: &mut Vec<LintError>) {
    // Stack of (env_name, line_number)
    let mut stack: Vec<(&str, usize)> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('%') {
            continue;
        }

        for env in extract_commands(trimmed, "begin") {
            stack.push((env, line_num));
        }

        for env in extract_commands(trimmed, "end") {
            if let Some((open_env, _)) = stack.last() {
                if *open_env == env {
                    stack.pop();
                } else {
                    errors.push(LintError {
                        file: rel.to_string(),
                        line: line_num,
                        message: format!("\\end{{{}}} does not match \\begin{{{}}}", env, open_env),
                        suggestion: Some(format!("Expected \\end{{{}}}", open_env)),
                    });
                }
            } else {
                errors.push(LintError {
                    file: rel.to_string(),
                    line: line_num,
                    message: format!("\\end{{{}}} without matching \\begin", env),
                    suggestion: None,
                });
            }
        }
    }

    // Report unclosed environments
    for (env, line_num) in stack {
        errors.push(LintError {
            file: rel.to_string(),
            line: line_num,
            message: format!("\\begin{{{}}} never closed", env),
            suggestion: Some(format!("Add \\end{{{}}}", env)),
        });
    }
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
                let arg = after[1..end].trim();
                if !arg.is_empty() {
                    results.push(arg);
                }
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

/// Recursively collect .tex files referenced by `\input{}`.
fn collect_tex_files(
    root: &Path,
    entry: &str,
    files: &mut Vec<std::path::PathBuf>,
    errors: &mut Vec<LintError>,
) {
    let path = resolve_tex_path(root, entry);
    if !path.exists() {
        return;
    }
    if files.contains(&path) {
        errors.push(LintError {
            file: entry.to_string(),
            line: 0,
            message: format!("Circular \\input detected: {}", path.display()),
            suggestion: Some("Remove the circular reference".into()),
        });
        return;
    }
    files.push(path.clone());

    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            let line = strip_comment(line);
            for input in extract_commands(&line, "input") {
                collect_tex_files(root, input, files, errors);
            }
        }
    }
}

/// Strip LaTeX comment from a line (everything after unescaped %).
fn strip_comment(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut prev_backslash = false;

    for c in line.chars() {
        if c == '%' && !prev_backslash {
            break;
        }
        prev_backslash = c == '\\';
        result.push(c);
    }

    result
}

/// Parse `@type{key, ...}` entries from a .bib file.
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
