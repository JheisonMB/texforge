//! LaTeX code formatter.
//!
//! Opinionated formatter inspired by `rustfmt` — one canonical output
//! regardless of input style.

const INDENT: &str = "  ";

/// Environments whose content must not be modified.
const VERBATIM_ENVS: &[&str] = &["verbatim", "lstlisting", "minted", "Verbatim"];

/// Format LaTeX source code with consistent style.
pub fn format(source: &str) -> String {
    let mut output = Vec::new();
    let mut depth: usize = 0;
    let mut prev_blank = false;
    let mut verbatim: Option<String> = None;

    for line in source.lines() {
        // Inside verbatim: pass through untouched until matching \end
        if let Some(ref env) = verbatim {
            let end_tag = format!("\\end{{{}}}", env);
            if line.trim().starts_with(&end_tag) {
                verbatim = None;
                depth = depth.saturating_sub(1);
                output.push(line.trim().to_string());
            } else {
                output.push(line.to_string());
            }
            continue;
        }

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

        let indented = if depth > 0
            && !trimmed.starts_with("\\begin{")
            && !trimmed.starts_with("\\end{")
            && !is_structural_command(trimmed)
        {
            format!("{}{}", INDENT.repeat(depth), trimmed)
        } else {
            trimmed.to_string()
        };

        output.push(indented);

        // Indent after \begin{...} and check for verbatim
        if trimmed.starts_with("\\begin{") {
            if let Some(env) = extract_env_name(trimmed) {
                if VERBATIM_ENVS.contains(&env.as_str()) {
                    verbatim = Some(env);
                }
            }
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

/// Commands that should not be indented even inside environments.
fn is_structural_command(line: &str) -> bool {
    const STRUCTURAL: &[&str] = &[
        "\\begin{",
        "\\end{",
        "\\documentclass",
        "\\usepackage",
        "\\section",
        "\\subsection",
        "\\chapter",
        "\\title",
        "\\author",
        "\\date",
        "\\maketitle",
        "\\tableofcontents",
        "\\input",
        "\\bibliography",
        "\\bibliographystyle",
        "\\newcommand",
        "\\renewcommand",
        "\\pagestyle",
        "\\geometry",
        "\\hypersetup",
        "\\numberwithin",
        "\\titleformat",
        "\\titlespacing",
        "\\fancyhf",
        "\\cfoot",
    ];
    STRUCTURAL.iter().any(|cmd| line.starts_with(cmd))
}

/// Extract environment name from `\begin{envname}`.
fn extract_env_name(line: &str) -> Option<String> {
    let start = line.find("\\begin{")? + 7;
    let end = line[start..].find('}')?;
    Some(line[start..start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_single_newline() {
        assert_eq!(format(""), "\n");
    }

    #[test]
    fn trailing_newline_always_present() {
        let out = format("hello");
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn indentation_inside_environment() {
        let src = "\\begin{document}\nhello\n\\end{document}";
        let out = format(src);
        assert!(out.contains("  hello"), "expected indented 'hello', got:\n{}", out);
    }

    #[test]
    fn multiple_blank_lines_collapsed() {
        let src = "a\n\n\n\nb";
        let out = format(src);
        assert_eq!(out, "a\n\nb\n");
    }

    #[test]
    fn structural_commands_not_indented() {
        let src = "\\begin{document}\n\\section{Intro}\n\\end{document}";
        let out = format(src);
        assert!(out.contains("\n\\section{Intro}\n"), "got:\n{}", out);
    }

    #[test]
    fn verbatim_content_preserved() {
        let src = "\\begin{verbatim}\n  raw   content\n\\end{verbatim}";
        let out = format(src);
        assert!(out.contains("  raw   content"), "got:\n{}", out);
    }
}
