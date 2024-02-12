use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use log::debug;

use crate::RuntimeConfig;

/// Spawn Memos server.
///
/// Spawns a managed child process with custom environment variables.
pub fn spawn(rconfig: &RuntimeConfig) -> Result<()> {
    let mut env_vars: HashMap<String, String> = prepare_env(rconfig);
    env_vars.extend(get_minimal_env());

    debug!("Memos environment: {:#?}", env_vars);

    let command = rconfig.paths.memos_bin.to_string_lossy().to_string();
    let cwd = get_cwd(rconfig);
    debug!("Memos server working directory: {}", cwd.to_string_lossy());
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

/// Decide which working directory to use for Memos server.
///
/// Front-end is not embedded in v0.18.2+ and Memos expects to
/// find the `dist` folder in its working directory.
///
/// On Linux, it will fail to access a `dist` folder under /usr/bin (where Tauri places the binary),
/// so we place the front-end in the data directory instead and change the working directory to there.
pub fn get_cwd(rconfig: &RuntimeConfig) -> PathBuf {
    if cfg!(dev) {
        return rconfig.paths.memospot_cwd.clone();
    }

    let mut search_paths: Vec<PathBuf> = vec![
        rconfig.paths.memospot_data.clone(),
        rconfig.paths.memospot_cwd.clone(),
    ];
    search_paths.dedup();

    // Prefer user-provided working_dir for Memos if it's not empty or ".".
    if let Some(working_dir) = &rconfig.yaml.memos.working_dir {
        let yaml_wd = working_dir.as_str().trim();
        if !yaml_wd.is_empty() {
            search_paths.insert(0, PathBuf::from(yaml_wd));
        }
    }

    for path in search_paths {
        let dist = path.join("dist");
        if dist.exists() && dist.is_dir() {
            return path;
        }
    }

    rconfig.paths.memospot_data.clone()
}

/// Make environment variable key suitable for Memos server.
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
    let managed_vars: HashMap<&str, String> = HashMap::from_iter(vec![
        ("mode", rconfig.yaml.memos.mode.to_string()),
        ("addr", rconfig.yaml.memos.addr.to_string()),
        ("port", rconfig.yaml.memos.port.to_string()),
        ("data", memos_data.to_string()),
        ("metric", rconfig.yaml.memos.metric.to_string()),
    ]);

    let mut env_vars: HashMap<String, String> = HashMap::new();

    // Add user's environment variables
    if let Some(memos_env) = &rconfig.yaml.memos.env {
        for (key, value) in memos_env {
            env_vars.insert(make_env_key(key), value.into());
        }
    }

    // Add managed environment variables.
    for (key, value) in managed_vars {
        env_vars.insert(make_env_key(key), value);
    }
    env_vars
}

/// Filter out system environment variables that Memos (and many other console programs) doesn't need.
///
/// Used after passing `.env_clear()` to `spawn`.
pub fn get_minimal_env() -> HashMap<String, String> {
    let minimal_vars = [
        "PATH",
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
