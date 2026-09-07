use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tauri::Manager;

const PREFERENCES_FILE: &str = "preferences.json";

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Preferences {
    file: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapState {
    file: Option<String>,
}

#[derive(Serialize)]
struct DictionaryFile {
    path: String,
    content: String,
}

fn load_preferences(path: &Path) -> Preferences {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn bootstrap(app: tauri::AppHandle) -> Result<BootstrapState, String> {
    let preferences_path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(PREFERENCES_FILE);
    let preferences = load_preferences(&preferences_path);
    let file = preferences.file.filter(|file| Path::new(file).is_file());
    Ok(BootstrapState { file })
}

#[tauri::command]
fn read_dictionary(path: String) -> Result<DictionaryFile, String> {
    if !path.ends_with(".dict.yaml") {
        return Err("只能打开 .dict.yaml 文件".into());
    }
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    Ok(DictionaryFile { path, content })
}

#[tauri::command]
fn remember_selection(app: tauri::AppHandle, file: String) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
    let serialized = serde_json::to_string_pretty(&Preferences { file: Some(file) })
        .map_err(|error| error.to_string())?;
    fs::write(config_dir.join(PREFERENCES_FILE), serialized).map_err(|error| error.to_string())
}

fn safe_save(path: &Path, content: &str) -> Result<(), String> {
    if !path.is_file()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".dict.yaml"))
    {
        return Err("目标必须是现有的 .dict.yaml 文件".into());
    }
    let backup = PathBuf::from(format!("{}.backup", path.display()));
    fs::copy(path, &backup).map_err(|error| error.to_string())?;
    AtomicFile::new(path, OverwriteBehavior::AllowOverwrite)
        .write(|file| file.write_all(content.as_bytes()))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_dictionary(path: String, content: String) -> Result<(), String> {
    safe_save(Path::new(&path), &content)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            read_dictionary,
            remember_selection,
            save_dictionary
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Rime Dict Studio");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_save_overwrites_one_rolling_backup() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sample.dict.yaml");
        fs::write(&path, "first").unwrap();
        safe_save(&path, "second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(
            fs::read_to_string(format!("{}.backup", path.display())).unwrap(),
            "first"
        );
        safe_save(&path, "third").unwrap();
        assert_eq!(
            fs::read_to_string(format!("{}.backup", path.display())).unwrap(),
            "second"
        );
    }
}
