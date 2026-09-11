use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryGroup {
    root_path: String,
    files: Vec<DictionaryFile>,
    warnings: Vec<String>,
}

#[derive(Deserialize)]
struct DictionaryWrite {
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

fn imported_tables(content: &str) -> Result<Vec<String>, String> {
    let header = content
        .lines()
        .take_while(|line| line.trim() != "...")
        .collect::<Vec<_>>()
        .join("\n");
    let value = serde_yaml::from_str::<serde_yaml::Value>(&header)
        .map_err(|error| format!("invalid-yaml-header: {error}"))?;
    Ok(value
        .get("import_tables")
        .and_then(serde_yaml::Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(serde_yaml::Value::as_str)
        .map(str::to_owned)
        .collect())
}

fn dictionary_path(directory: &Path, table: &str) -> PathBuf {
    let name = if table.ends_with(".dict.yaml") {
        table.to_string()
    } else {
        format!("{table}.dict.yaml")
    };
    directory.join(name)
}

fn read_group_file(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    files: &mut Vec<DictionaryFile>,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }
    let content = fs::read_to_string(&canonical).map_err(|error| error.to_string())?;
    let parent = canonical
        .parent()
        .ok_or_else(|| "missing-parent-directory".to_string())?;
    let imports = imported_tables(&content)?;
    files.push(DictionaryFile {
        path: canonical.to_string_lossy().into_owned(),
        content,
    });
    for table in imports {
        let imported = dictionary_path(parent, &table);
        if imported.is_file() {
            read_group_file(&imported, visited, files, warnings)?;
        } else {
            log::warn!("missing imported dictionary: {}", imported.display());
            warnings.push(imported.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

#[tauri::command]
fn read_dictionary_group(path: String) -> Result<DictionaryGroup, String> {
    if !path.ends_with(".dict.yaml") {
        return Err("invalid-dictionary-extension".into());
    }
    let root = Path::new(&path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    read_group_file(&root, &mut HashSet::new(), &mut files, &mut warnings)?;
    log::info!(
        "loaded dictionary group: {} files, {} warnings",
        files.len(),
        warnings.len()
    );
    Ok(DictionaryGroup {
        root_path: root.to_string_lossy().into_owned(),
        files,
        warnings,
    })
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

#[tauri::command]
fn save_dictionaries(writes: Vec<DictionaryWrite>) -> Result<(), String> {
    safe_save_many(&writes)
}

fn write_atomic(path: &Path, content: &[u8]) -> Result<(), String> {
    AtomicFile::new(path, OverwriteBehavior::AllowOverwrite)
        .write(|file| file.write_all(content))
        .map_err(|error| error.to_string())
}

fn safe_save_many(writes: &[DictionaryWrite]) -> Result<(), String> {
    safe_save_many_with(writes, write_atomic)
}

fn safe_save_many_with<F>(writes: &[DictionaryWrite], mut writer: F) -> Result<(), String>
where
    F: FnMut(&Path, &[u8]) -> Result<(), String>,
{
    let mut paths = HashSet::new();
    let originals = writes
        .iter()
        .map(|write| {
            let path = PathBuf::from(&write.path);
            if !path.is_file()
                || !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".dict.yaml"))
                || !paths.insert(path.clone())
            {
                return Err("invalid-or-duplicate-dictionary-path".to_string());
            }
            fs::read(&path)
                .map(|content| (path, content))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    for (path, original) in &originals {
        writer(Path::new(&format!("{}.backup", path.display())), original)?;
    }
    for (index, write) in writes.iter().enumerate() {
        if let Err(error) = writer(Path::new(&write.path), write.content.as_bytes()) {
            let mut rollback_errors = Vec::new();
            for (path, original) in originals.iter().take(index) {
                if let Err(rollback_error) = writer(path, original) {
                    log::error!("failed to roll back {}: {}", path.display(), rollback_error);
                    rollback_errors.push(format!("{}: {rollback_error}", path.display()));
                }
            }
            return if rollback_errors.is_empty() {
                Err(error)
            } else {
                Err(format!(
                    "{error}; rollback-failed: {}",
                    rollback_errors.join(", ")
                ))
            };
        }
    }
    log::info!("saved dictionary group changes: {} files", writes.len());
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            read_dictionary,
            read_dictionary_group,
            remember_selection,
            save_dictionary,
            save_dictionaries
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

    #[test]
    fn parses_only_active_import_tables() {
        let content = "---\nimport_tables:\n  - main # active\n  # - disabled\n  - 'extra'\nother: value\n...\n";
        assert_eq!(imported_tables(content).unwrap(), vec!["main", "extra"]);
        assert_eq!(
            imported_tables("---\nimport_tables: [main, extra]\n...\n").unwrap(),
            vec!["main", "extra"]
        );
        assert_eq!(
            imported_tables("---\ndescription: \"...\"\nimport_tables: [main]\n...\n").unwrap(),
            vec!["main"]
        );
    }

    #[test]
    fn reads_recursive_group_once_and_warns_for_missing_imports() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root.dict.yaml");
        let child = directory.path().join("child.dict.yaml");
        fs::write(
            &root,
            "---\nimport_tables:\n  - child\n  - absent\n...\n甲\ta\n",
        )
        .unwrap();
        fs::write(&child, "---\nimport_tables:\n  - root\n...\n乙\tb\n").unwrap();
        let group = read_dictionary_group(root.to_string_lossy().into_owned()).unwrap();
        assert_eq!(group.files.len(), 2);
        assert_eq!(group.warnings.len(), 1);
    }

    #[test]
    fn safely_saves_multiple_files_with_rolling_backups() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.dict.yaml");
        let second = directory.path().join("second.dict.yaml");
        fs::write(&first, "first-old").unwrap();
        fs::write(&second, "second-old").unwrap();
        safe_save_many(&[
            DictionaryWrite {
                path: first.to_string_lossy().into_owned(),
                content: "first-new".into(),
            },
            DictionaryWrite {
                path: second.to_string_lossy().into_owned(),
                content: "second-new".into(),
            },
        ])
        .unwrap();
        assert_eq!(fs::read_to_string(&first).unwrap(), "first-new");
        assert_eq!(fs::read_to_string(&second).unwrap(), "second-new");
        assert_eq!(
            fs::read_to_string(format!("{}.backup", first.display())).unwrap(),
            "first-old"
        );
        assert_eq!(
            fs::read_to_string(format!("{}.backup", second.display())).unwrap(),
            "second-old"
        );
    }

    #[test]
    fn rolls_back_an_earlier_file_when_a_later_write_fails() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.dict.yaml");
        let second = directory.path().join("second.dict.yaml");
        fs::write(&first, "first-old").unwrap();
        fs::write(&second, "second-old").unwrap();
        let writes = [
            DictionaryWrite {
                path: first.to_string_lossy().into_owned(),
                content: "first-new".into(),
            },
            DictionaryWrite {
                path: second.to_string_lossy().into_owned(),
                content: "second-new".into(),
            },
        ];
        let result = safe_save_many_with(&writes, |path, content| {
            if path == second && content == b"second-new" {
                return Err("simulated-write-error".into());
            }
            write_atomic(path, content)
        });
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&first).unwrap(), "first-old");
        assert_eq!(fs::read_to_string(&second).unwrap(), "second-old");
    }

    #[test]
    fn rejects_an_invalid_yaml_header() {
        assert!(imported_tables("---\nimport_tables: [broken\n...\n").is_err());
    }
}
