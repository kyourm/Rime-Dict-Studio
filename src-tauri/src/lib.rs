mod deployer;

use atomicwrites::{AtomicFile, OverwriteBehavior};
#[cfg(test)]
use deployer::Platform;
use deployer::{
    current_platform, detect_deployer, run_deployer, unavailable_state, DeploymentCandidate,
    DeploymentConfig, DeploymentContext, DeploymentState,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use tauri::Manager;

const PREFERENCES_FILE: &str = "preferences.json";

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Preferences {
    file: Option<String>,
    deployer: Option<DeploymentConfig>,
}

impl Preferences {
    fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }
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

fn preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|error| error.to_string())
        .map(|directory| directory.join(PREFERENCES_FILE))
}

fn save_preferences(path: &Path, preferences: &Preferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "missing-preferences-directory".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let serialized =
        serde_json::to_string_pretty(preferences).map_err(|error| error.to_string())?;
    fs::write(path, serialized).map_err(|error| error.to_string())
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

fn read_imported_tables(path: &Path) -> Result<Vec<String>, String> {
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut header = String::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim() == "..." {
            break;
        }
        header.push_str(&line);
        header.push('\n');
    }
    imported_tables(&header)
}

fn dictionary_path(directory: &Path, table: &str) -> PathBuf {
    let name = if table.ends_with(".dict.yaml") {
        table.to_string()
    } else {
        format!("{table}.dict.yaml")
    };
    directory.join(name)
}

fn discover_group_roots(selected: &Path) -> Vec<PathBuf> {
    let Some(directory) = selected.parent() else {
        return vec![selected.to_path_buf()];
    };
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            log::warn!(
                "failed to scan dictionary directory {}: {}",
                directory.display(),
                error
            );
            return vec![selected.to_path_buf()];
        }
    };
    let mut definitions = Vec::new();
    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                log::warn!("failed to inspect a dictionary directory entry: {error}");
                continue;
            }
        };
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".dict.yaml"))
        {
            continue;
        }
        let imports = match read_imported_tables(&path) {
            Ok(imports) => imports,
            Err(error) => {
                log::warn!(
                    "failed to parse dictionary header {}: {}",
                    path.display(),
                    error
                );
                continue;
            }
        };
        let Ok(canonical) = path.canonicalize() else {
            log::warn!("failed to resolve dictionary path: {}", path.display());
            continue;
        };
        let resolved = imports
            .into_iter()
            .map(|table| dictionary_path(directory, &table))
            .filter_map(|path| path.canonicalize().ok())
            .collect::<HashSet<_>>();
        definitions.push((canonical, resolved));
    }

    let mut ancestors = HashSet::from([selected.to_path_buf()]);
    loop {
        let parents = definitions
            .iter()
            .filter(|(path, imports)| {
                !ancestors.contains(path) && imports.iter().any(|import| ancestors.contains(import))
            })
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        if parents.is_empty() {
            break;
        }
        ancestors.extend(parents);
    }
    let mut roots = ancestors
        .iter()
        .filter(|candidate| {
            !definitions.iter().any(|(path, imports)| {
                ancestors.contains(path) && imports.contains(candidate.as_path())
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    roots.sort();
    if roots.is_empty() {
        vec![selected.to_path_buf()]
    } else {
        roots
    }
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
    let roots = discover_group_roots(&root);
    let mut visited = HashSet::new();
    for group_root in &roots {
        read_group_file(group_root, &mut visited, &mut files, &mut warnings)?;
    }
    log::info!(
        "loaded dictionary group from {} root(s): {} files, {} warnings",
        roots.len(),
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
    let path = preferences_path(&app)?;
    let preferences = load_preferences(&path).with_file(file);
    save_preferences(&path, &preferences)
}

fn deployment_context(
    app: &tauri::AppHandle,
    user_data_dir: PathBuf,
) -> Result<DeploymentContext, String> {
    let home = app.path().home_dir().map_err(|error| error.to_string())?;
    let program_roots = ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let path_entries = std::env::var_os("PATH")
        .map(|value| std::env::split_paths(&value).collect())
        .unwrap_or_default();
    let macos_candidates = vec![
        PathBuf::from("/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel"),
        home.join("Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel"),
    ];
    Ok(DeploymentContext {
        platform: current_platform(),
        program_roots,
        path_entries,
        user_data_dir,
        macos_candidates,
    })
}

fn resolve_deployer(
    app: &tauri::AppHandle,
    user_data_dir: PathBuf,
) -> Result<Option<DeploymentCandidate>, String> {
    let preferences = load_preferences(&preferences_path(app)?);
    if let Some(config) = preferences.deployer {
        return Ok(Some(DeploymentCandidate::from_custom(config)));
    }
    Ok(detect_deployer(&deployment_context(app, user_data_dir)?))
}

#[tauri::command]
fn deployment_state(
    app: tauri::AppHandle,
    user_directory: String,
) -> Result<DeploymentState, String> {
    Ok(resolve_deployer(&app, PathBuf::from(user_directory))?
        .map_or_else(unavailable_state, |candidate| candidate.state()))
}

#[tauri::command]
fn save_deployment_config(
    app: tauri::AppHandle,
    config: Option<DeploymentConfig>,
) -> Result<(), String> {
    if let Some(candidate) = &config {
        if !candidate.executable.is_file() {
            return Err("deployer-not-found".into());
        }
        if candidate
            .working_directory
            .as_ref()
            .is_some_and(|directory| !directory.is_dir())
        {
            return Err("deployer-working-directory-not-found".into());
        }
    }
    let path = preferences_path(&app)?;
    let mut preferences = load_preferences(&path);
    preferences.deployer = config;
    save_preferences(&path, &preferences)
}

#[tauri::command]
async fn deploy_rime(app: tauri::AppHandle, user_directory: String) -> Result<(), String> {
    let candidate = resolve_deployer(&app, PathBuf::from(user_directory))?
        .ok_or_else(|| "deployer-not-found".to_string())?;
    log::info!(
        "starting Rime deployment with {} ({})",
        candidate.label,
        candidate.executable.display()
    );
    let result = tauri::async_runtime::spawn_blocking(move || run_deployer(&candidate))
        .await
        .map_err(|error| format!("deployer-task-failed: {error}"))?;
    if let Err(error) = &result {
        log::error!("Rime deployment failed: {error}");
    } else {
        log::info!("Rime deployment completed");
    }
    result
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
            save_dictionaries,
            deployment_state,
            save_deployment_config,
            deploy_rime
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Rime Dict Studio");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_squirrel_and_preserves_the_selected_dictionary_as_context() {
        let directory = tempfile::tempdir().unwrap();
        let squirrel = directory.path().join("Squirrel");
        fs::write(&squirrel, "").unwrap();
        let context = DeploymentContext {
            platform: Platform::Macos,
            program_roots: vec![],
            path_entries: vec![],
            user_data_dir: directory.path().join("Rime"),
            macos_candidates: vec![squirrel.clone()],
        };

        let deployer = detect_deployer(&context).unwrap();

        assert_eq!(deployer.executable, squirrel);
        assert_eq!(deployer.arguments, vec!["--reload"]);
        assert!(!deployer.experimental);
    }

    #[test]
    fn detects_weasel_in_the_latest_standard_install_directory() {
        let directory = tempfile::tempdir().unwrap();
        let older = directory.path().join("Rime/weasel-0.9.0");
        let latest = directory.path().join("Rime/weasel-0.17.4");
        fs::create_dir_all(&older).unwrap();
        fs::create_dir_all(&latest).unwrap();
        fs::write(older.join("WeaselDeployer.exe"), "").unwrap();
        fs::write(latest.join("WeaselDeployer.exe"), "").unwrap();
        let context = DeploymentContext {
            platform: Platform::Windows,
            program_roots: vec![directory.path().to_path_buf()],
            path_entries: vec![],
            user_data_dir: directory.path().join("Rime"),
            macos_candidates: vec![],
        };

        let deployer = detect_deployer(&context).unwrap();

        assert_eq!(deployer.executable, latest.join("WeaselDeployer.exe"));
        assert_eq!(deployer.arguments, vec!["/deploy"]);
        assert_eq!(deployer.working_directory, Some(latest));
    }

    #[test]
    fn detects_linux_deployer_as_experimental() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("rime_deployer");
        fs::write(&executable, "").unwrap();
        let user_data_dir = directory.path().join("rime");
        let context = DeploymentContext {
            platform: Platform::Linux,
            program_roots: vec![],
            path_entries: vec![directory.path().to_path_buf()],
            user_data_dir: user_data_dir.clone(),
            macos_candidates: vec![],
        };

        let deployer = detect_deployer(&context).unwrap();

        assert!(deployer.experimental);
        assert_eq!(
            deployer.arguments.first().map(String::as_str),
            Some("--build")
        );
        assert_eq!(
            deployer.arguments.get(1).map(String::as_str),
            Some(user_data_dir.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn remembering_a_file_keeps_the_custom_deployer() {
        let preferences = Preferences {
            file: None,
            deployer: Some(DeploymentConfig {
                executable: "/custom/deployer".into(),
                arguments: vec!["--deploy".into()],
                working_directory: None,
            }),
        };

        let updated = preferences.with_file("/rime/user.dict.yaml".into());

        assert_eq!(updated.file.as_deref(), Some("/rime/user.dict.yaml"));
        assert_eq!(
            updated.deployer.unwrap().executable,
            PathBuf::from("/custom/deployer")
        );
    }

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
    fn discovers_the_group_when_an_imported_child_is_selected() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root.dict.yaml");
        let child = directory.path().join("child.dict.yaml");
        let sibling = directory.path().join("sibling.dict.yaml");
        fs::write(&root, "---\nimport_tables: [child, sibling]\n...\n根\tr\n").unwrap();
        fs::write(&child, "---\nname: child\n...\n子\tc\n").unwrap();
        fs::write(&sibling, "---\nname: sibling\n...\n旁\ts\n").unwrap();

        let group = read_dictionary_group(child.to_string_lossy().into_owned()).unwrap();

        assert_eq!(
            group.root_path,
            child.canonicalize().unwrap().to_string_lossy()
        );
        assert_eq!(group.files.len(), 3);
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
