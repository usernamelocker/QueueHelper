use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::{
    core::{event_bus::EventBus, events::AppEvent, state::RuntimeState},
    lcu::types::LockfileInfo,
};

pub fn parse_lockfile(raw: &str, lockfile_path: &Path) -> Result<LockfileInfo> {
    let values: Vec<&str> = raw.trim().split(':').collect();
    if values.len() != 5 {
        return Err(anyhow!(
            "invalid lockfile format (expected 5 fields, got {})",
            values.len()
        ));
    }

    let process_name = values[0].to_string();
    let pid = values[1]
        .parse::<u32>()
        .context("failed parsing lockfile pid")?;
    let port = values[2]
        .parse::<u16>()
        .context("failed parsing lockfile port")?;
    let password = values[3].to_string();
    let protocol = values[4].to_string();

    Ok(LockfileInfo {
        process_name,
        pid,
        port,
        password,
        protocol,
        lockfile_path: lockfile_path.to_path_buf(),
    })
}

pub async fn run_lockfile_monitor(
    bus: EventBus,
    shared_state: std::sync::Arc<RwLock<RuntimeState>>,
    shutdown: CancellationToken,
) {
    let mut previous_signature: Option<String> = None;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = sleep(Duration::from_secs(1)) => {}
        }

        let override_path = {
            let state = shared_state.read().await;
            state.settings.league_install_path.clone()
        };

        let discovered_path = discover_lockfile_path(override_path.as_deref());
        if let Some(lockfile_path) = discovered_path {
            match tokio::fs::read_to_string(&lockfile_path).await {
                Ok(raw) => match parse_lockfile(&raw, &lockfile_path) {
                    Ok(lockfile) => {
                        let signature =
                            format!("{}:{}:{}", lockfile_path.display(), lockfile.port, lockfile.pid);
                        if previous_signature.as_ref() != Some(&signature) {
                            previous_signature = Some(signature);
                            bus.publish(AppEvent::LockfileDetected(lockfile));
                        }
                    }
                    Err(error) => {
                        bus.publish(AppEvent::Monitor {
                            level: crate::models::MonitorLevel::Warn,
                            category: "lockfile".to_string(),
                            message: format!("Failed to parse lockfile: {}", error),
                        });
                    }
                },
                Err(error) => {
                    bus.publish(AppEvent::Monitor {
                        level: crate::models::MonitorLevel::Warn,
                        category: "lockfile".to_string(),
                        message: format!("Failed to read lockfile: {}", error),
                    });
                }
            }
            continue;
        }

        if previous_signature.take().is_some() {
            bus.publish(AppEvent::LockfileMissing);
        }
    }
}

fn discover_lockfile_path(override_path: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = override_path {
        let candidate = normalize_lockfile_path(PathBuf::from(path));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    for candidate in common_lockfile_candidates() {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn normalize_lockfile_path(path: PathBuf) -> PathBuf {
    if path
        .file_name()
        .and_then(|filename| filename.to_str())
        .map(|filename| filename.eq_ignore_ascii_case("lockfile"))
        .unwrap_or(false)
    {
        return path;
    }
    path.join("lockfile")
}

fn common_lockfile_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    candidates.push(PathBuf::from(r"C:\Riot Games\League of Legends\lockfile"));
    candidates.push(PathBuf::from(
        r"C:\Program Files\Riot Games\League of Legends\lockfile",
    ));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\Riot Games\League of Legends\lockfile",
    ));

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("Riot Games")
                .join("League of Legends")
                .join("lockfile"),
        );
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("Riot Games")
                .join("League of Legends")
                .join("lockfile"),
        );
    }

    candidates
}

