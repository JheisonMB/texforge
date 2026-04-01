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

    // Mirror asset files so tectonic resolves relative paths
    crate::utils::mirror_assets(root, &build_dir)?;

    Ok(build_dir.join(entry))
}

/// Replace all `\begin{mermaid}[opts]...\end{mermaid}` with figure environments.
fn render_diagrams(content: &str, diagrams_dir: &Path, counter: &mut usize) -> Result<String> {
    let content = render_env(content, "mermaid", diagrams_dir, counter, |src| {
        let svg = mermaid_rs_renderer::render(src)
            .map_err(|e| anyhow::anyhow!("Mermaid render error: {}", e))?;
        svg_to_png(&svg).context("Failed to convert mermaid SVG to PNG")
    })?;
    let content = render_env(&content, "graphviz", diagrams_dir, counter, |src| {
        let svg = render_graphviz(src)?;
        svg_to_png(&svg).context("Failed to convert graphviz SVG to PNG")
    })?;
    Ok(content)
}

/// Generic environment renderer: replaces `\begin{env}[opts]...\end{env}` with figure.
pub(crate) fn render_env(
    content: &str,
    env: &str,
    diagrams_dir: &Path,
    counter: &mut usize,
    render_fn: impl Fn(&str) -> Result<Vec<u8>>,
) -> Result<String> {
    let begin_tag = format!("\\begin{{{}}}", env);
    let end_tag = format!("\\end{{{}}}", env);

    let mut result = String::new();
    let mut remaining: &str = content;

    while let Some(start) = remaining.find(&begin_tag) {
        result.push_str(&remaining[..start]);

        let after_begin = &remaining[start + begin_tag.len()..];
        let (opts, after_opts) = parse_opts(after_begin);

        let end = after_opts
            .find(&*end_tag)
            .with_context(|| format!("\\begin{{{}}} without matching \\end{{{}}}", env, env))?;

        let diagram_src = after_opts[..end].trim();

        // Fail fast: validate pos before doing any rendering work
        let pos = opts.get("pos").map(String::as_str).unwrap_or("H");
        if !["H", "t", "b", "h", "p"].contains(&pos) {
            anyhow::bail!(
                "Invalid {} option pos='{}' — valid values are: H, t, b, h, p",
                env,
                pos
            );
        }

        let png = render_fn(diagram_src)?;

        *counter += 1;
        let filename = format!("diagram-{}.png", counter);
        std::fs::write(diagrams_dir.join(&filename), &png)?;

        // Build figure environment
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
        remaining = &after_opts[end + end_tag.len()..];
    }

    result.push_str(remaining);
    Ok(result)
}

/// Render a DOT/Graphviz diagram to SVG using layout-rs (pure Rust).
fn render_graphviz(src: &str) -> Result<String> {
    use layout::backends::svg::SVGWriter;
    use layout::gv::DotParser;
    use layout::gv::GraphBuilder;
    use layout::topo::layout::VisualGraph;

    let mut parser = DotParser::new(src);
    let graph = parser.process().map_err(|e| {
        parser.print_error();
        anyhow::anyhow!("Graphviz parse error: {}", e)
    })?;

    let mut builder = GraphBuilder::new();
    builder.visit_graph(&graph);
    let mut vg: VisualGraph = builder.get();

    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);
    Ok(svg.finalize())
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

/// Collect .tex files reachable from entry via \input.
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

    #[test]
    fn render_graphviz_produces_svg() {
        let dot = "digraph G { A -> B }";
        let svg = render_graphviz(dot).unwrap();
        assert!(
            svg.contains("<svg"),
            "expected SVG output, got: {}",
            &svg[..100.min(svg.len())]
        );
    }

    #[test]
    fn render_env_no_blocks_unchanged() {
        let content = "hello world";
        let dir = tempfile::tempdir().unwrap();
        let mut counter = 0;
        let result = render_env(content, "graphviz", dir.path(), &mut counter, |_| {
            Ok(vec![])
        })
        .unwrap();
        assert_eq!(result, content);
        assert_eq!(counter, 0);
    }

    #[test]
    fn render_env_invalid_pos_returns_error() {
        let content = "\\begin{graphviz}[pos=Z]\ndigraph G{}\n\\end{graphviz}";
        let dir = tempfile::tempdir().unwrap();
        let mut counter = 0;
        let err = render_env(content, "graphviz", dir.path(), &mut counter, |_| {
            Ok(vec![1, 2, 3])
        })
        .unwrap_err();
        assert!(err.to_string().contains("pos='Z'"));
    }
}
