//! Template manifest parser and schema validation.
//!
//! Parses `template.toml` files that describe LaTeX templates with metadata,
//! placeholders, and post-generation scripts.

#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Template manifest structure matching the Phase 1 spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,

    #[serde(default)]
    pub files: FileSpec,

    #[serde(default)]
    pub placeholders: Vec<Placeholder>,

    #[serde(default)]
    pub post_generate: Vec<PostGenerateScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSpec {
    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,
}

/// A placeholder that can be substituted during template generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    pub name: String,
    pub r#type: PlaceholderType,
    pub description: String,

    #[serde(default)]
    pub required: bool,

    #[serde(default)]
    pub default: Option<String>,

    #[serde(default)]
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlaceholderType {
    String,
    Boolean,
    Enum,
}

/// A post-generate script that the user can optionally run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostGenerateScript {
    pub name: String,
    pub description: String,
    pub command: String,

    #[serde(default)]
    pub optional: bool,
}

impl TemplateManifest {
    /// Parse a template.toml file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read template.toml: {}", path.display()))?;

        Self::from_str(&content)
    }

    /// Parse from TOML string
    pub fn from_str(content: &str) -> Result<Self> {
        let manifest: TemplateManifest = toml::from_str(content)
            .context("Failed to parse template.toml — invalid TOML syntax")?;

        // Validate required fields
        if manifest.id.is_empty() {
            return Err(anyhow!("Template manifest: id is required"));
        }
        if manifest.version.is_empty() {
            return Err(anyhow!("Template manifest: version is required"));
        }
        if manifest.display_name.is_empty() {
            return Err(anyhow!("Template manifest: display_name is required"));
        }

        // Validate placeholders
        let mut placeholder_names = std::collections::HashSet::new();
        for ph in &manifest.placeholders {
            if ph.name.is_empty() {
                return Err(anyhow!(
                    "Template manifest: placeholder name cannot be empty"
                ));
            }
            if !placeholder_names.insert(&ph.name) {
                return Err(anyhow!(
                    "Template manifest: duplicate placeholder name '{}'",
                    ph.name
                ));
            }

            // Enum type must have choices
            if ph.r#type == PlaceholderType::Enum && ph.choices.is_none() {
                return Err(anyhow!(
                    "Template manifest: enum placeholder '{}' must have 'choices'",
                    ph.name
                ));
            }
        }

        Ok(manifest)
    }

    /// Get a placeholder by name
    pub fn get_placeholder(&self, name: &str) -> Option<&Placeholder> {
        self.placeholders.iter().find(|p| p.name == name)
    }

    /// Get all required placeholders
    pub fn required_placeholders(&self) -> Vec<&Placeholder> {
        self.placeholders.iter().filter(|p| p.required).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_manifest() {
        let toml = r#"
id = "basic-article"
version = "1.0.0"
display_name = "Basic Article"
description = "A simple article template"

[[placeholders]]
name = "title"
type = "string"
description = "Document title"
required = true
"#;
        let manifest = TemplateManifest::from_str(toml).unwrap();
        assert_eq!(manifest.id, "basic-article");
        assert_eq!(manifest.placeholders.len(), 1);
        assert!(manifest.get_placeholder("title").is_some());
    }

    #[test]
    fn test_parse_with_default() {
        let toml = r#"
id = "test"
version = "1.0.0"
display_name = "Test"
description = "Test template"

[[placeholders]]
name = "author"
type = "string"
description = "Author name"
default = "{{user.name}}"
"#;
        let manifest = TemplateManifest::from_str(toml).unwrap();
        let author = manifest.get_placeholder("author").unwrap();
        assert_eq!(author.default, Some("{{user.name}}".to_string()));
    }

    #[test]
    fn test_enum_placeholder_requires_choices() {
        let toml = r#"
id = "test"
version = "1.0.0"
display_name = "Test"
description = "Test"

[[placeholders]]
name = "language"
type = "enum"
description = "Document language"
"#;
        let result = TemplateManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have 'choices'"));
    }

    #[test]
    fn test_duplicate_placeholder_names() {
        let toml = r#"
id = "test"
version = "1.0.0"
display_name = "Test"
description = "Test"

[[placeholders]]
name = "title"
type = "string"
description = "Title"

[[placeholders]]
name = "title"
type = "string"
description = "Duplicate"
"#;
        let result = TemplateManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate"));
    }
}
