//! Utility functions.

use std::path::Path;

/// Get the TexForge data directory (~/.texforge)
pub fn data_dir() -> anyhow::Result<std::path::PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let data_dir = home.join(".texforge");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

/// Get the templates directory (~/.texforge/templates)
pub fn templates_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = data_dir()?.join("templates");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Find all .tex files in a directory
pub fn find_tex_files(root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    
    for entry in walkdir::WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "tex" {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }
    
    Ok(files)
}
