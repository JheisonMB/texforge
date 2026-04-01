//! Utility functions.

use std::path::Path;

use anyhow::Context;

/// Get the `TexForge` data directory (~/.texforge)
pub fn data_dir() -> anyhow::Result<std::path::PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
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

/// Find all .tex files in a directory, excluding build/
pub fn find_tex_files(root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let build_dir = root.join("build");

    for entry in walkdir::WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.starts_with(&build_dir) {
            continue;
        }
        if entry.file_type().is_file() {
            if let Some(ext) = path.extension() {
                if ext == "tex" {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Mirror asset directories into build/ using symlinks (Unix) or file copy (Windows).
/// Skips .tex files (handled by the diagram pre-processor) and build/ itself.
pub fn mirror_assets(root: &Path, build_dir: &Path) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') || path == build_dir {
            continue;
        }

        let dest = build_dir.join(&name);

        if path.is_dir() {
            if dest.exists() || dest.symlink_metadata().is_ok() {
                continue;
            }
            link_or_copy_dir(&path, &dest)?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "tex" && !dest.exists() {
                std::fs::copy(&path, &dest)?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn link_or_copy_dir(src: &Path, dest: &Path) -> anyhow::Result<()> {
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
fn link_or_copy_dir(src: &Path, dest: &Path) -> anyhow::Result<()> {
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
