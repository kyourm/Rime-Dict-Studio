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
    directory: Option<String>,
    file: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapState {
    directory: Option<String>,
    file: Option<String>,
    dictionaries: Vec<String>,
}

#[derive(Serialize)]
struct DictionaryFile {
    path: String,
    content: String,
}

fn default_rime_directories() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if cfg!(target_os = "windows") {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            candidates.push(PathBuf::from(app_data).join("Rime"));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(home).join("Library/Rime"));
        }
    } else if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join(".config/ibus/rime"));
        candidates.push(home.join(".local/share/fcitx5/rime"));
    }
    candidates
}

fn dictionary_paths(directory: &Path) -> Result<Vec<String>, String> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<String> = fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".dict.yaml"))
        })
        .filter_map(|path| path.to_str().map(str::to_owned))
        .collect();
    paths.sort();
    Ok(paths)
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
    let remembered = preferences
        .directory
        .as_ref()
        .map(PathBuf::from)
        .filter(|path| path.is_dir());
    let directory = remembered.or_else(|| {
        default_rime_directories()
            .into_iter()
            .find(|path| path.is_dir())
    });
    let dictionaries = directory
        .as_deref()
        .map(dictionary_paths)
        .transpose()?
        .unwrap_or_default();
    let file = preferences
        .file
        .filter(|file| Path::new(file).is_file() && dictionaries.contains(file));
    Ok(BootstrapState {
        directory: directory.and_then(|path| path.to_str().map(str::to_owned)),
        file,
        dictionaries,
    })
}

#[tauri::command]
fn list_dictionaries(directory: String) -> Result<Vec<String>, String> {
    dictionary_paths(Path::new(&directory))
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
fn remember_selection(
    app: tauri::AppHandle,
    directory: String,
    file: String,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
    let serialized = serde_json::to_string_pretty(&Preferences {
        directory: Some(directory),
        file: Some(file),
    })
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
            list_dictionaries,
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
