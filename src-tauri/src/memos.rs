use itertools::Itertools;
use log::{debug, info};
use memospot::absolute_path;
use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

use crate::RuntimeConfig;

/// Spawn Memos server.
///
/// Spawns a managed child process with custom environment variables.
pub fn spawn(rconfig: &RuntimeConfig) -> Result<()> {
    let mut env_vars: HashMap<String, String> = prepare_env(rconfig);
    env_vars.extend(get_minimal_env());

    debug!("Memos's environment: {:#?}", env_vars);

    let command = rconfig.paths.memos_bin.to_string_lossy().to_string();
    let cwd = get_cwd(rconfig);
    info!("Memos's working directory: {}", cwd.to_string_lossy());
    tauri::async_runtime::spawn(async move {
        tauri::api::process::Command::new(command)
            .env_clear()
            .envs(env_vars)
            .current_dir(cwd.clone())
            .spawn()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    });
    Ok(())
}

/// Decide which working directory use for Memos's server.
///
/// Front-end is not embedded in v0.18.2+ and Memos expects to
/// find the `dist` folder in its working directory.
///
/// On Linux, Memos will fail to access a `dist` folder under /usr/bin
/// (where Tauri places the binary), so we look for the `dist` folder
/// following this order of precedence:
/// 1. User-provided working directory from the yaml file.
/// 2. Tauri's resource directory.
/// 3. Memospot's data directory.
/// 4. Memospot's current working directory.
///
/// Finally, if no `dist` folder is found, fall back to Memospot's data directory.
pub fn get_cwd(rconfig: &RuntimeConfig) -> PathBuf {
    let mut search_paths: Vec<PathBuf> = Vec::new();

    // Prefer user-provided working_dir, if it's not empty.
    if let Some(working_dir) = &rconfig.yaml.memos.working_dir {
        let yaml_wd = working_dir.as_str().trim();
        if !yaml_wd.is_empty() {
            let path = absolute_path(Path::new(yaml_wd)).unwrap_or_default();
            search_paths.push(path);
        }
    }
    let binding = rconfig
        .paths
        ._memospot_resources
        .as_os_str()
        .to_string_lossy();

    // Tauri uses `canonicalize()` to resolve the resource directory,
    // which adds a `\\?\` prefix on Windows.
    let resources = binding.trim_start_matches("\\\\?\\");

    search_paths.extend(Vec::from([
        PathBuf::from(resources),
        rconfig.paths.memospot_data.clone(),
        rconfig.paths.memospot_cwd.clone(),
    ]));

    let deduped: Vec<PathBuf> = search_paths.into_iter().unique().collect();
    debug!("Looking for Memos's `dist` folder at {:#?}", deduped);

    for path in deduped {
        if path.as_os_str().is_empty() {
            continue;
        }
        let dist = path.join("dist");
        if dist.exists() && dist.is_dir() {
            return path;
        }
    }

    rconfig.paths.memospot_data.clone()
}

/// Make environment variable key suitable for Memos's server.
fn make_env_key(key: &str) -> String {
    let uppercased_key = key.to_uppercase();
    if uppercased_key.starts_with("MEMOS_") {
        return uppercased_key;
    }
    format!("MEMOS_{}", uppercased_key)
}

/// Prepare environment variables for Memos server.
pub fn prepare_env(rconfig: &RuntimeConfig) -> HashMap<String, String> {
    // Use the runtime-checked memos_data variable instead of the one from the yaml file.
    let memos_data = rconfig.paths.memos_data.to_string_lossy();
    let yaml = rconfig.yaml.memos.clone();
    let managed_vars: HashMap<&str, String> = HashMap::from_iter(vec![
        ("mode", yaml.mode.unwrap_or_default()),
        ("addr", yaml.addr.unwrap_or_default()),
        ("port", yaml.port.unwrap_or_default().to_string()),
        ("data", memos_data.to_string()),
        ("metric", yaml.metric.unwrap_or_default().to_string()),
    ]);

    let mut env_vars: HashMap<String, String> = HashMap::new();

    // Add user's environment variables
    if let Some(memos_env) = &rconfig.yaml.memos.env {
        for (key, value) in memos_env {
            env_vars.insert(make_env_key(key), value.into());
        }
    }

    // Add managed environment variables. The default insert()
    // behavior will overwrite the value of an existing key.
    for (key, value) in managed_vars {
        env_vars.insert(make_env_key(key), value);
    }
    env_vars
}

/// Filter out system environment variables that Memos doesn't need.
///
/// Used after passing `.env_clear()` to `spawn`.
pub fn get_minimal_env() -> HashMap<String, String> {
    let minimal_vars = [
        // "PATH",
        "PWD",
        "SHELL",
        "SHLVL",
        "TERM",
        "TZ",
        "PROGRAMDATA",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
    ];
    env::vars_os()
        .filter(|(key, _)| {
            let k = key.to_string_lossy().to_uppercase();
            minimal_vars.contains(&k.as_str())
        })
        .map(|(key, value)| {
            (
                key.into_string().unwrap(),
                value.into_string().unwrap_or_default(),
            )
        })
        .collect()
}

// pub fn spawn(bin: &PathBuf, env_vars: &HashMap<String, String>) -> Result<()> {
//     let command = bin.clone().to_string_lossy().to_string();
//     tauri::async_runtime::spawn(async move {
//         let Ok(cmd) = tauri::api::process::Command::new(command)
//             .envs(memos_env_vars)
//             .current_dir(memos_cwd)
//             .spawn()
//         else {
//             panic_dialog!("Failed to spawn Memos server!");
//         };

//         if current_config.memos.log.enabled {
//             // log levels are: trace, debug, info, warn, error, off
//             let memos_log = memospot_data.clone().join("memos.log");
//             let log_level = &current_config.memos.log.level.clone().to_lowercase();

//             let Ok(mut file) = tokio::fs::OpenOptions::new()
//                 .create(true)
//                 .append(true)
//                 .open(&memos_log)
//                 .await
//             else {
//                 panic_dialog!(
//                     "Failed to open log file for writing:\n{}",
//                     &memos_log.to_string_lossy().to_string()
//                 );
//             };

//             let (mut rx, _) = cmd;
//             while let Some(event) = rx.recv().await {
//                 match event {
//                     CommandEvent::Stdout(line) => {
//                         if !["trace", "debug"].contains(&log_level.as_str()) {
//                             continue;
//                         }
//                         if line.is_empty() {
//                             continue;
//                         }
//                         if let Err(e) = file.write_all(line.as_bytes()).await {
//                             error!(
//                                 "Failed to write log to file:\n{}\n\n{}",
//                                 &memos_log.to_string_lossy().to_string(),
//                                 e
//                             );
//                         }
//                     }
//                     CommandEvent::Stderr(line) => {
//                         if line.is_empty() {
//                             continue;
//                         }
//                         if let Err(e) = file.write_all(line.as_bytes()).await {
//                             error!(
//                                 "Failed to write log to file:\n{}\n\n{}",
//                                 &memos_log.to_string_lossy().to_string(),
//                                 e
//                             );
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     });

//     let mut cmd = Command::new(bin);
//     cmd.envs(env_vars);
//     cmd.spawn()
//         .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
// }
