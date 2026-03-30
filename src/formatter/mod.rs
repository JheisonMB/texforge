//! LaTeX code formatter.
//!
//! Opinionated formatter inspired by `rustfmt` — one canonical output
//! regardless of input style.

const INDENT: &str = "  ";

/// Format LaTeX source code with consistent style.
pub fn format(source: &str) -> String {
    let mut output = Vec::new();
    let mut depth: usize = 0;
    let mut prev_blank = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Collapse multiple blank lines into one
        if trimmed.is_empty() {
            if !prev_blank && !output.is_empty() {
                output.push(String::new());
            }
            prev_blank = true;
            continue;
        }
        prev_blank = false;

        // Dedent for \end{...}
        if trimmed.starts_with("\\end{") {
            depth = depth.saturating_sub(1);
        }

        let indented = if depth > 0 && !trimmed.starts_with('%') && !trimmed.starts_with('\\') {
            // Content lines inside environments get indented
            format!("{}{}", INDENT.repeat(depth), trimmed)
        } else if depth > 0
            && trimmed.starts_with('\\')
            && !trimmed.starts_with("\\begin{")
            && !trimmed.starts_with("\\end{")
            && !trimmed.starts_with("\\documentclass")
            && !trimmed.starts_with("\\usepackage")
            && !trimmed.starts_with("\\section")
            && !trimmed.starts_with("\\subsection")
            && !trimmed.starts_with("\\chapter")
            && !trimmed.starts_with("\\title")
            && !trimmed.starts_with("\\author")
            && !trimmed.starts_with("\\date")
            && !trimmed.starts_with("\\maketitle")
            && !trimmed.starts_with("\\tableofcontents")
            && !trimmed.starts_with("\\input")
            && !trimmed.starts_with("\\bibliography")
            && !trimmed.starts_with("\\bibliographystyle")
            && !trimmed.starts_with("\\newcommand")
            && !trimmed.starts_with("\\renewcommand")
            && !trimmed.starts_with("\\pagestyle")
            && !trimmed.starts_with("\\geometry")
            && !trimmed.starts_with("\\hypersetup")
            && !trimmed.starts_with("\\numberwithin")
            && !trimmed.starts_with("\\titleformat")
            && !trimmed.starts_with("\\titlespacing")
            && !trimmed.starts_with("\\fancyhf")
            && !trimmed.starts_with("\\cfoot")
        {
            format!("{}{}", INDENT.repeat(depth), trimmed)
        } else {
            trimmed.to_string()
        };

        output.push(indented);

        // Indent after \begin{...}
        if trimmed.starts_with("\\begin{") {
            depth += 1;
        }
    }

    // Remove trailing blank lines
    while output.last().is_some_and(|l| l.is_empty()) {
        output.pop();
    }

    let mut result = output.join("\n");
    result.push('\n');
    result
}
