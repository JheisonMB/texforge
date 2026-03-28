//! Static linting rules.

/// Linter for LaTeX projects
pub struct Linter;

impl Linter {
    /// Run all linting rules on project
    pub fn lint(_project_root: &std::path::Path) -> anyhow::Result<Vec<LintError>> {
        // TODO: Implement linting rules
        Ok(Vec::new())
    }
}

/// A linting error with location and suggestion
#[derive(Debug)]
pub struct LintError {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub suggestion: Option<String>,
}
