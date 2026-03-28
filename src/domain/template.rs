//! Template metadata and management.

use serde::{Deserialize, Serialize};

/// Template metadata from template.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub metadata: MetadataSection,
    pub variables: VariablesSection,
    pub compatibilidad: CompatibilidadSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSection {
    pub nombre: String,
    pub descripcion: String,
    pub idioma: String,
    pub tipo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablesSection {
    pub requeridas: Vec<String>,
    #[serde(default)]
    pub opcionales: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilidadSection {
    pub mermaid: bool,
    pub graphviz: bool,
    pub bibliografia: String,
}

/// Represents a LaTeX template
#[derive(Debug)]
pub struct Template {
    pub name: String,
    pub metadata: TemplateMetadata,
    pub preambulo: String,
    pub portada: Option<String>,
}
