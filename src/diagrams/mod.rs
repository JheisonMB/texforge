//! Pre-processor for embedded diagram environments.
//!
//! Intercepts `\begin{mermaid}...\end{mermaid}` blocks before tectonic sees
//! the file, renders them to SVG, and replaces each block with
//! `\includegraphics{build/diagrams/diagram-N.svg}`.

use std::path::Path;

use anyhow::{Context, Result};

/// Process all .tex files reachable from `entry`, rendering embedded diagrams.
/// Returns true if any diagrams were rendered (so the caller knows files changed).
pub fn process(root: &Path, entry: &str) -> Result<bool> {
    let entry_path = root.join(entry);
    let content = std::fs::read_to_string(&entry_path)?;

    let diagrams_dir = root.join("build/diagrams");
    std::fs::create_dir_all(&diagrams_dir)?;

    let mut any = false;
    any |= process_file(root, &entry_path, &diagrams_dir)?;

    // Also process \input-ted files
    for line in content.lines() {
        for input in extract_inputs(line) {
            let path = resolve_tex(root, input);
            if path.exists() {
                any |= process_file(root, &path, &diagrams_dir)?;
            }
        }
    }

    Ok(any)
}

fn process_file(root: &Path, path: &Path, diagrams_dir: &Path) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    if !content.contains("\\begin{mermaid}") {
        return Ok(false);
    }

    let (new_content, count) = render_diagrams(&content, diagrams_dir)
        .with_context(|| format!("Failed to render diagrams in {}", path.display()))?;

    if count > 0 {
        std::fs::write(path, &new_content)?;
        eprintln!("  rendered {} mermaid diagram(s) in {}", count, path.strip_prefix(root).unwrap_or(path).display());
    }

    Ok(count > 0)
}

/// Replace all `\begin{mermaid}...\end{mermaid}` blocks with `\includegraphics`.
fn render_diagrams(content: &str, diagrams_dir: &Path) -> Result<(String, usize)> {
    let mut result = String::new();
    let mut remaining: &str = &content;
    let mut count = 0;

    while let Some(start) = remaining.find("\\begin{mermaid}") {
        result.push_str(&remaining[..start]);

        let after_begin = &remaining[start + "\\begin{mermaid}".len()..];
        let end = after_begin
            .find("\\end{mermaid}")
            .context("\\begin{mermaid} without matching \\end{mermaid}")?;

        let diagram_src = after_begin[..end].trim();

        // Render to PNG (LaTeX-compatible without extra packages)
        let svg_str = mermaid_rs_renderer::render(diagram_src)
            .map_err(|e| anyhow::anyhow!("Mermaid render error: {}", e))?;
        let png = svg_to_png(&svg_str)
            .context("Failed to convert mermaid SVG to PNG")?;

        // Save PNG file
        let filename = format!("diagram-{}.png", count + 1);
        let img_path = diagrams_dir.join(&filename);
        std::fs::write(&img_path, &png)?;

        // Replace with \includegraphics
        let rel = format!("build/diagrams/{}", filename);
        result.push_str(&format!("\\includegraphics{{{}}}", rel));

        remaining = &after_begin[end + "\\end{mermaid}".len()..];
        count += 1;
    }

    result.push_str(remaining);
    Ok((result, count))
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

fn resolve_tex(root: &Path, input: &str) -> std::path::PathBuf {
    let p = root.join(input);
    if p.extension().is_some() { p } else { p.with_extension("tex") }
}

/// Convert SVG string to PNG bytes using resvg (2x scale for print quality).
fn svg_to_png(svg: &str) -> Result<Vec<u8>> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &options)
        .context("Failed to parse SVG")?;

    let scale = 2.0_f32;
    let width = (tree.size().width() * scale) as u32;
    let height = (tree.size().height() * scale) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .context("Failed to create pixmap")?;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().context("Failed to encode PNG")
}
