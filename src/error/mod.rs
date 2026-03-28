//! Custom error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TexForgeError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
