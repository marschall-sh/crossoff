use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config;
use crate::model::AppData;

fn data_file_path() -> PathBuf {
    let cfg = config::load();
    if let Some(custom) = cfg.data_dir {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            // Backward-compatible behavior: allow either
            // - a directory path (we store <dir>/data.json)
            // - a file path ending in .json
            if p.extension().and_then(|e| e.to_str()) == Some("json") {
                return p;
            }
            return p.join("data.json");
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("crossoff").join("data.json")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/crossoff/data.json")
    }
}

fn backup_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(format!("{}.bak", path.display()))
}

fn temp_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(format!("{}.tmp", path.display()))
}

pub fn load() -> Result<AppData> {
    let path = data_file_path();
    let backup_path = backup_path(&path);

    if path.exists() {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        match serde_json::from_str::<AppData>(&contents) {
            Ok(data) => return Ok(data),
            Err(primary_err) => {
                if backup_path.exists() {
                    let backup_contents = fs::read_to_string(&backup_path).with_context(|| {
                        format!("failed to read backup {}", backup_path.display())
                    })?;
                    let backup_data = serde_json::from_str::<AppData>(&backup_contents)
                        .with_context(|| {
                            format!(
                                "failed to parse {} and backup {}",
                                path.display(),
                                backup_path.display()
                            )
                        })?;
                    return Ok(backup_data);
                }
                return Err(primary_err).with_context(|| {
                    format!("failed to parse {}", path.display())
                });
            }
        }
    }

    if backup_path.exists() {
        let backup_contents = fs::read_to_string(&backup_path)
            .with_context(|| format!("failed to read backup {}", backup_path.display()))?;
        let backup_data = serde_json::from_str::<AppData>(&backup_contents)
            .with_context(|| format!("failed to parse backup {}", backup_path.display()))?;
        return Ok(backup_data);
    }

    Ok(AppData::default())
}

pub fn save(data: &AppData) -> Result<()> {
    let path = data_file_path();
    let backup_path = backup_path(&path);
    let temp_path = temp_path(&path);

    let dir = path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&dir)?;

    let contents = serde_json::to_string_pretty(data)?;

    // Keep the last known-good file as a fallback.
    if path.exists() {
        fs::copy(&path, &backup_path).with_context(|| {
            format!(
                "failed to create backup {} from {}",
                backup_path.display(),
                path.display()
            )
        })?;
    }

    // Atomic save: write temp file in same directory, then rename into place.
    fs::write(&temp_path, contents)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    fs::rename(&temp_path, &path).with_context(|| {
        format!(
            "failed to atomically replace {} with {}",
            path.display(),
            temp_path.display()
        )
    })?;

    Ok(())
}
