use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const DEPLOY_TIMEOUT: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Platform {
    Windows,
    Macos,
    Linux,
    Unsupported,
}

#[derive(Clone, Debug)]
pub(crate) struct DeploymentContext {
    pub platform: Platform,
    pub program_roots: Vec<PathBuf>,
    pub path_entries: Vec<PathBuf>,
    pub user_data_dir: PathBuf,
    pub macos_candidates: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeploymentConfig {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct DeploymentCandidate {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub label: String,
    pub experimental: bool,
    pub custom: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeploymentState {
    pub available: bool,
    pub label: String,
    pub executable: Option<String>,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub experimental: bool,
    pub custom: bool,
}

impl DeploymentCandidate {
    pub(crate) fn from_custom(config: DeploymentConfig) -> Self {
        Self {
            executable: config.executable,
            arguments: config.arguments,
            working_directory: config.working_directory,
            label: "custom".into(),
            experimental: false,
            custom: true,
        }
    }

    pub(crate) fn state(&self) -> DeploymentState {
        DeploymentState {
            available: self.executable.is_file(),
            label: self.label.clone(),
            executable: Some(self.executable.to_string_lossy().into_owned()),
            arguments: self.arguments.clone(),
            working_directory: self
                .working_directory
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            experimental: self.experimental,
            custom: self.custom,
        }
    }
}

pub(crate) fn unavailable_state() -> DeploymentState {
    DeploymentState {
        available: false,
        label: "unavailable".into(),
        executable: None,
        arguments: Vec::new(),
        working_directory: None,
        experimental: false,
        custom: false,
    }
}

pub(crate) fn current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::Macos
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else {
        Platform::Unsupported
    }
}

pub(crate) fn detect_deployer(context: &DeploymentContext) -> Option<DeploymentCandidate> {
    match context.platform {
        Platform::Macos => context
            .macos_candidates
            .iter()
            .find(|path| path.is_file())
            .map(|executable| DeploymentCandidate {
                executable: executable.clone(),
                arguments: vec!["--reload".into()],
                working_directory: executable.parent().map(Path::to_path_buf),
                label: "squirrel".into(),
                experimental: false,
                custom: false,
            }),
        Platform::Windows => detect_weasel(context),
        Platform::Linux => find_in_path("rime_deployer", &context.path_entries).map(|executable| {
            let shared_data = PathBuf::from("/usr/share/rime-data");
            let shared_data = if shared_data.is_dir() {
                shared_data
            } else {
                context.user_data_dir.clone()
            };
            DeploymentCandidate {
                executable,
                arguments: vec![
                    "--build".into(),
                    context.user_data_dir.to_string_lossy().into_owned(),
                    shared_data.to_string_lossy().into_owned(),
                    context
                        .user_data_dir
                        .join("build")
                        .to_string_lossy()
                        .into_owned(),
                ],
                working_directory: Some(context.user_data_dir.clone()),
                label: "librime".into(),
                experimental: true,
                custom: false,
            }
        }),
        Platform::Unsupported => None,
    }
}

fn detect_weasel(context: &DeploymentContext) -> Option<DeploymentCandidate> {
    let mut version_directories = context
        .program_roots
        .iter()
        .map(|root| root.join("Rime"))
        .filter_map(|root| fs::read_dir(root).ok())
        .flat_map(|entries| entries.filter_map(Result::ok).map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("weasel-"))
        })
        .collect::<Vec<_>>();
    version_directories.sort_by_key(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(version_numbers)
            .unwrap_or_default()
    });
    version_directories.reverse();
    version_directories
        .into_iter()
        .find_map(|working_directory| {
            let executable = working_directory.join("WeaselDeployer.exe");
            executable.is_file().then(|| DeploymentCandidate {
                executable,
                arguments: vec!["/deploy".into()],
                working_directory: Some(working_directory),
                label: "weasel".into(),
                experimental: false,
                custom: false,
            })
        })
}

fn version_numbers(name: &str) -> Vec<u32> {
    name.split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

fn find_in_path(name: &str, entries: &[PathBuf]) -> Option<PathBuf> {
    entries
        .iter()
        .map(|entry| entry.join(name))
        .find(|path| path.is_file())
}

pub(crate) fn run_deployer(candidate: &DeploymentCandidate) -> Result<(), String> {
    run_deployer_with_timeout(candidate, DEPLOY_TIMEOUT)
}

fn run_deployer_with_timeout(
    candidate: &DeploymentCandidate,
    timeout: Duration,
) -> Result<(), String> {
    if !candidate.executable.is_file() {
        return Err("deployer-not-found".into());
    }
    let mut command = Command::new(&candidate.executable);
    command
        .args(&candidate.arguments)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(directory) = &candidate.working_directory {
        command.current_dir(directory);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("deployer-start-failed: {error}"))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return status
                .success()
                .then_some(())
                .ok_or_else(|| format!("deployer-exit-failed: {}", status.code().unwrap_or(-1)));
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err("deployer-timeout".into());
        }
        thread::sleep(POLL_INTERVAL.min(timeout));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn reports_a_non_zero_exit_code() {
        let candidate = DeploymentCandidate {
            executable: PathBuf::from("/bin/sh"),
            arguments: vec!["-c".into(), "exit 7".into()],
            working_directory: None,
            label: "test".into(),
            experimental: false,
            custom: true,
        };

        assert_eq!(
            run_deployer_with_timeout(&candidate, Duration::from_secs(1)),
            Err("deployer-exit-failed: 7".into())
        );
    }

    #[cfg(unix)]
    #[test]
    fn terminates_a_timed_out_deployer() {
        let candidate = DeploymentCandidate {
            executable: PathBuf::from("/bin/sh"),
            arguments: vec!["-c".into(), "sleep 1".into()],
            working_directory: None,
            label: "test".into(),
            experimental: false,
            custom: true,
        };

        assert_eq!(
            run_deployer_with_timeout(&candidate, Duration::from_millis(10)),
            Err("deployer-timeout".into())
        );
    }
}
