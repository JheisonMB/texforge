//! Pre-processor for embedded diagram environments.
//!
//! Intercepts `\begin{mermaid}[opts]...\end{mermaid}` blocks, renders them
//! to PNG, and replaces each block with a proper `figure` environment.
//!
//! Works on copies in `build/` — the original .tex files are never modified.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Copy all .tex files to `build/`, rendering embedded diagrams in the copies.
/// Also mirrors non-.tex assets so tectonic can resolve relative paths.
/// Returns the path to the build copy of `entry`.
pub fn process(root: &Path, entry: &str) -> Result<PathBuf> {
    let build_dir = root.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let diagrams_dir = build_dir.join("diagrams");
    std::fs::create_dir_all(&diagrams_dir)?;

    let mut counter = 0usize;

    // Process .tex files
    let tex_files = collect_tex_files(root, entry);
    for src in &tex_files {
        let rel = src.strip_prefix(root).unwrap_or(src);
        let dest = build_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = std::fs::read_to_string(src)?;
        let processed = render_diagrams(&content, &diagrams_dir, &mut counter)
            .with_context(|| format!("Failed to render diagrams in {}", src.display()))?;
        std::fs::write(&dest, processed)?;
    }

    // Mirror asset files (non-.tex, non-build) so tectonic resolves relative paths
    mirror_assets(root, &build_dir)?;

    Ok(build_dir.join(entry))
}

/// Replace all `\begin{mermaid}[opts]...\end{mermaid}` with figure environments.
fn render_diagrams(content: &str, diagrams_dir: &Path, counter: &mut usize) -> Result<String> {
    let mut result = String::new();
    let mut remaining: &str = content;

    while let Some(start) = remaining.find("\\begin{mermaid}") {
        result.push_str(&remaining[..start]);

        let after_begin = &remaining[start + "\\begin{mermaid}".len()..];

        // Parse optional args: \begin{mermaid}[key=val, ...]
        let (opts, after_opts) = parse_opts(after_begin);

        let end = after_opts
            .find("\\end{mermaid}")
            .context("\\begin{mermaid} without matching \\end{mermaid}")?;

        let diagram_src = after_opts[..end].trim();

        // Render SVG → PNG
        let svg = mermaid_rs_renderer::render(diagram_src)
            .map_err(|e| anyhow::anyhow!("Mermaid render error: {}", e))?;
        let png = svg_to_png(&svg).context("Failed to convert mermaid SVG to PNG")?;

        *counter += 1;
        let filename = format!("diagram-{}.png", counter);
        std::fs::write(diagrams_dir.join(&filename), &png)?;

        // Build figure environment
        let pos = opts.get("pos").map(String::as_str).unwrap_or("H");
        if !["H", "t", "b", "h", "p"].contains(&pos) {
            anyhow::bail!(
                "Invalid mermaid option pos='{}' — valid values are: H, t, b, h, p",
                pos
            );
        }
        let width = opts
            .get("width")
            .map(String::as_str)
            .unwrap_or("\\linewidth");
        let caption = opts.get("caption");
        let rel_path = format!("diagrams/{}", filename);

        let mut fig = format!(
            "\\begin{{figure}}[{pos}]\n  \\centering\n  \\includegraphics[width={width}]{{{rel_path}}}\n"
        );
        if let Some(cap) = caption {
            fig.push_str(&format!("  \\caption{{{}}}\n", cap));
        }
        fig.push_str("\\end{figure}");

        result.push_str(&fig);
        remaining = &after_opts[end + "\\end{mermaid}".len()..];
    }

    result.push_str(remaining);
    Ok(result)
}

/// Parse `[key=val, key2=val2]` into a map. Returns `(map, rest_of_str)`.
pub(crate) fn parse_opts(s: &str) -> (HashMap<String, String>, &str) {
    let s = s.trim_start_matches('\n').trim_start_matches('\r');
    if !s.starts_with('[') {
        return (HashMap::new(), s);
    }
    let Some(end) = s.find(']') else {
        return (HashMap::new(), s);
    };
    let inner = &s[1..end];
    let rest = &s[end + 1..];

    let mut map = HashMap::new();
    for part in inner.split(',') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    (map, rest)
}

/// Collect .tex files reachable from entry via \input (non-recursive for simplicity).
fn collect_tex_files(root: &Path, entry: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(root, entry, &mut files);
    files
}

fn collect_recursive(root: &Path, entry: &str, files: &mut Vec<PathBuf>) {
    let path = resolve_tex(root, entry);
    if !path.exists() || files.contains(&path) {
        return;
    }
    files.push(path.clone());
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            for input in extract_inputs(line) {
                collect_recursive(root, input, files);
            }
        }
    }
}

fn extract_inputs(line: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut search = line;
    while let Some(pos) = search.find("\\input{") {
        let after = &search[pos + 7..];
        if let Some(end) = after.find('}') {
            results.push(after[..end].trim());
            search = &after[end + 1..];
        } else {
            break;
        }
    }
    results
}

fn resolve_tex(root: &Path, input: &str) -> PathBuf {
    let p = root.join(input);
    if p.extension().is_some() {
        p
    } else {
        p.with_extension("tex")
    }
}

/// Mirror asset directories into build/ using symlinks (Unix) or file copy (Windows).
/// Skips .tex files (handled separately) and the build/ dir itself.
fn mirror_assets(root: &Path, build_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden, build/, and .tex files at root level
        if name_str.starts_with('.') || path == build_dir {
            continue;
        }

        let dest = build_dir.join(&name);

        if path.is_dir() {
            // Remove stale symlink/dir if it points somewhere wrong
            if dest.exists() || dest.symlink_metadata().is_ok() {
                continue; // already linked
            }
            link_or_copy_dir(&path, &dest)?;
        }
        // Individual root-level files (e.g. .bib at root) — skip .tex
        else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "tex" {
                continue;
            }
            if !dest.exists() {
                std::fs::copy(&path, &dest)?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn link_or_copy_dir(src: &Path, dest: &Path) -> Result<()> {
    // Symlink: dest -> ../dirname (relative from build/)
    let target = std::path::Path::new("..").join(src.file_name().unwrap());
    std::os::unix::fs::symlink(&target, dest).with_context(|| {
        format!(
            "Failed to symlink {} -> {}",
            dest.display(),
            target.display()
        )
    })
}

#[cfg(not(unix))]
fn link_or_copy_dir(src: &Path, dest: &Path) -> Result<()> {
    // Windows fallback: recursive copy
    copy_dir_recursive(src, dest)
}

#[cfg(not(unix))]
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let rel = entry.path().strip_prefix(src).unwrap();
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Convert SVG string to PNG bytes at 2x scale for print quality.
fn svg_to_png(svg: &str) -> Result<Vec<u8>> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &options).context("Failed to parse SVG")?;

    let scale = 2.0_f32;
    let width = (tree.size().width() * scale) as u32;
    let height = (tree.size().height() * scale) as u32;

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().context("Failed to encode PNG")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_opts_no_brackets_returns_empty_map() {
        let (map, rest) = parse_opts("hello");
        assert!(map.is_empty());
        assert_eq!(rest, "hello");
    }

    #[test]
    fn parse_opts_width_and_pos() {
        let (map, _) = parse_opts("[width=0.5\\linewidth, pos=t]");
        assert_eq!(map.get("width").map(String::as_str), Some("0.5\\linewidth"));
        assert_eq!(map.get("pos").map(String::as_str), Some("t"));
    }

    #[test]
    fn parse_opts_caption() {
        let (map, _) = parse_opts("[caption=My diagram]");
        assert_eq!(map.get("caption").map(String::as_str), Some("My diagram"));
    }
}
