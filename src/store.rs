use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::model::AppData;

fn data_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("crossoff")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/crossoff")
    }
}

pub fn load() -> Result<AppData> {
    let path = data_dir().join("data.json");
    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let data: AppData = serde_json::from_str(&contents)?;
        Ok(data)
    } else {
        Ok(AppData::default())
    }
}

pub fn save(data: &AppData) -> Result<()> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    let contents = serde_json::to_string_pretty(data)?;
    fs::write(dir.join("data.json"), contents)?;
    Ok(())
}
