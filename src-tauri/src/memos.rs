use crate::utils::absolute_path;
use crate::{process, RuntimeConfig};
use homedir::HomeDirExt;
use itertools::Itertools;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use tauri_plugin_http::reqwest;

/// Spawn Memos server.
///
/// Spawns a managed child process with custom environment variables.
pub fn spawn(rtcfg: &RuntimeConfig) -> Result<()> {
    let env_vars: HashMap<String, String> = prepare_env(rtcfg);
    let command = rtcfg.paths.memos_bin.to_string_lossy().to_string();
    let cwd = get_cwd(rtcfg);
    debug!("Memos environment: {:#?}", env_vars);
    debug!("Memos working directory: {}", cwd.to_string_lossy());
    tauri::async_runtime::spawn(async move {
        process::Command::new(command)
            .envs(env_vars)
            .current_dir(cwd.clone())
            .spawn()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    });
    Ok(())
}

/// Decide which working directory use for Memos server.
///
/// The front end is not embedded from Memos v0.18.2 to v0.21.0,
/// and it expects to find the `dist` folder in its working directory.
///
/// On Linux, Memos will fail to access a `dist` folder under /usr/bin
/// (where Tauri places the binary), so we look for the `dist` folder
/// following this order of precedence:
/// 1. User-provided working directory from the yaml file.
/// 2. Tauri resource directory.
/// 3. Memospot data directory.
/// 4. Memospot current working directory.
///
/// Finally, if no `dist` folder is found, fall back to Memospot data directory.
pub fn get_cwd(rtcfg: &RuntimeConfig) -> PathBuf {
    let mut search_paths: Vec<PathBuf> = Vec::new();

    // Prefer user-provided working_dir, if it's not empty.
    if let Some(working_dir) = &rtcfg.yaml.memos.working_dir {
        let yaml_wd = working_dir.as_str().trim();
        if !yaml_wd.is_empty() {
            let expanded_path = Path::new(yaml_wd).expand_home().unwrap_or_default();
            let path = absolute_path(expanded_path).unwrap_or_default();
            search_paths.push(path);
        }
    }
    let binding = rtcfg
        .paths
        ._memospot_resources
        .as_os_str()
        .to_string_lossy();

    // Tauri uses `canonicalize()` to resolve the resource directory,
    // which adds a `\\?\` prefix on Windows.
    let resources = binding.trim_start_matches(r#"\\?\"#);

    search_paths.extend(Vec::from([
        PathBuf::from(resources),
        rtcfg.paths.memospot_data.clone(),
        rtcfg.paths.memospot_cwd.clone(),
    ]));

    let deduplicated: Vec<PathBuf> = search_paths.into_iter().unique().collect();
    debug!("Looking for Memos `dist` folder at {:#?}", deduplicated);

    for path in deduplicated {
        if path.as_os_str().is_empty() {
            continue;
        }
        let dist = path.join("dist");
        if dist.exists() && dist.is_dir() {
            return path;
        }
    }

    rtcfg.paths.memospot_data.clone()
}

/// Make environment variable key suitable for Memos server.
fn make_env_key(key: &str) -> String {
    let uppercased_key = key.to_uppercase().replace("-", "_");
    if uppercased_key.starts_with("MEMOS_") {
        return uppercased_key;
    }
    format!("MEMOS_{}", uppercased_key)
}

/// Prepare environment variables for Memos server.
pub fn prepare_env(rtcfg: &RuntimeConfig) -> HashMap<String, String> {
    // Use the runtime-checked memos_data variable instead of the one from the yaml file.
    let memos_data = rtcfg.paths.memos_data.to_string_lossy();
    let yaml = rtcfg.yaml.memos.clone();
    let managed_vars: HashMap<&str, String> = HashMap::from_iter(vec![
        ("mode", yaml.mode.unwrap_or_default()),
        ("addr", yaml.addr.unwrap_or_default()),
        ("port", yaml.port.unwrap_or_default().to_string()),
        ("data", memos_data.to_string()),
        // Metrics were removed from Memos v0.20+.
        ("metric", "false".to_string()),
        ("instance_url", rtcfg.memos_url.to_string()),
        // These were added in v0.22.4 and then removed. Sane defaults are hardcoded to prevent user lock-out.
        ("public", "false".to_string()),
        ("password_auth", "true".to_string()),
    ]);

    let mut env_vars: HashMap<String, String> = HashMap::new();

    // Add user's environment variables
    if let Some(memos_env) = &rtcfg.yaml.memos.env {
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

/// Query Memos version via API.
///
/// Working with Memos v0.23.0+.
pub async fn query_version(memos_url: &str) -> Result<String> {
    let endpoint = format!("{}api/v1/workspace/profile", memos_url);
    let url = match reqwest::Url::parse(&endpoint) {
        Ok(url) => url,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to parse Memos URL: {}", e),
            ));
        }
    };
    let client = reqwest::Client::new();
    let request = client
        .get(url)
        .header("User-Agent", "Memospot")
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .await;

    match request {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("server responded with status code: {}", response.status()),
                ));
            }
            let json = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            let version = json
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            Ok(version.to_string())
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

/// Poll Memos server until the API responds.
///
/// Currently, it just logs the server version.
///
/// This is a blocking function.
pub async fn wait_api_ready(memos_url: &str, interval_millis: u64, timeout_millis: u128) {
    let mut version = String::new();
    let mut last_error = String::new();
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_millis(interval_millis));
    let time_start = tokio::time::Instant::now();

    loop {
        if time_start.elapsed().as_millis() > timeout_millis {
            break;
        }
        interval.tick().await;
        match query_version(memos_url).await {
            Ok(v) => {
                version = v;
                break;
            }
            Err(e) => last_error = e.to_string(),
        }
    }

    if version.is_empty() {
        warn!(
            "Failed to query Memos version via API: {}. Giving up after {} ms.",
            last_error, timeout_millis
        );
        return;
    }
    info!(
        "Memos version: {}. API ready in <{} ms.",
        version,
        time_start.elapsed().as_millis()
    );
}
