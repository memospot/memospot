use crate::memos_log;
use crate::utils::absolute_path;
use crate::{sqlite, RuntimeConfig};
use anyhow::{anyhow, Result};
use dialog::ExpectDialogExt;
use homedir::HomeDirExt;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};
use tauri::utils::platform::resource_dir as tauri_resource_dir;
use tauri_plugin_http::reqwest;
use tauri_utils::PackageInfo;

#[derive(Clone, Debug, Default)]
pub struct VersionStore {
    version: Arc<Mutex<String>>,
}
impl VersionStore {
    /// Returns a reference to the global singleton instance of `VersionStore`.
    fn instance() -> &'static Self {
        static INSTANCE: LazyLock<VersionStore> = LazyLock::new(Default::default);
        &INSTANCE
    }
    /// Get version previously stored by [`memos::wait_api_ready()`].
    pub fn get() -> String {
        VersionStore::instance()
            .version
            .lock()
            .expect_dialog("unable to lock version store")
            .clone()
    }
    pub fn set(version: impl Into<String>) {
        let mut store = VersionStore::instance()
            .version
            .lock()
            .expect_dialog("unable to lock version store");
        *store = version.into();
    }
}

type MemosLaunchRetries = Arc<Mutex<i32>>;
fn memos_launch_retries() -> &'static MemosLaunchRetries {
    static LAUNCH_RETRIES: LazyLock<MemosLaunchRetries> = LazyLock::new(Default::default);
    &LAUNCH_RETRIES
}

/// Spawn Memos server.
///
/// Spawns a managed child process with custom environment variables.
pub fn spawn(rtcfg: &RuntimeConfig) -> Result<(), anyhow::Error> {
    let env_vars: HashMap<String, String> = prepare_env(rtcfg);
    let command = rtcfg.paths.memos_bin.to_string_lossy().to_string();
    let cwd = get_cwd(rtcfg);
    debug!("working directory: {}", cwd.to_string_lossy());
    debug!("environment: {env_vars:#?}");

    let res = sidecar::Command::new(&command)
        .envs(env_vars.clone())
        .current_dir(cwd.clone())
        .spawn();

    match res {
        Err(e) => {
            warn!("failed to spawn server: {e}.");

            #[cfg(unix)]
            {
                warn!("attempting to add executable permissions to the binary…");
                let mut perms = fs::metadata(&command)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&command, perms)?;

                warn!("permissions added successfully. Attempting to relaunch.");

                let mut attempts = memos_launch_retries()
                    .lock()
                    .expect_dialog("unable to acquire lock");

                if *attempts >= 1 {
                    warn!("exceeded launch retries limit ({attempts}/1). Giving up.");
                    return Err(e);
                }
                *attempts += 1;

                return spawn(rtcfg);
            }
            #[cfg(not(unix))]
            return Err(e);
        }
        Ok(receiver) => {
            if rtcfg.yaml.memospot.log.enabled.unwrap_or_default() {
                let (rx_, _) = receiver;
                tauri::async_runtime::spawn(async move {
                    memos_log::log_events(rx_).await;
                });
            }
        }
    }

    Ok(())
}

/// Shutdown the Memos server and checkpoint the database.
pub fn shutdown() {
    let config = RuntimeConfig::from_global_store();

    debug!("shutting down server…");
    sidecar::kill_children();

    let db_file = config.paths.memos_db_file.clone();
    tauri::async_runtime::block_on(async move {
        sqlite::wait_checkpoint(&db_file).await;
    });

    debug!("server shutdown");
}

/// Decide which working directory use for Memos server.
///
/// The front end is not embedded from Memos v0.18.2 to v0.21.0,
/// and it expects to find the `dist` folder in its working directory.
///
/// On Linux, Memos will fail to access a `dist` folder under /usr/bin
/// (where Tauri places the binary), so we look for the `dist` folder
/// following this order of precedence:
/// 1. User-provided working directory from the YAML configuration file.
/// 2. Tauri resource directory.
/// 3. Memospot data directory.
/// 4. Memospot current working directory.
///
/// Finally, if no `dist` folder is found, fall back to Memospot data directory.
pub fn get_cwd(rtcfg: &RuntimeConfig) -> PathBuf {
    let mut search_paths: Vec<PathBuf> = Vec::new();

    // Prefer user-provided working_dir, if it's not empty.
    if let Some(working_dir) = &rtcfg.yaml.memos.working_dir {
        if !working_dir.trim().is_empty() {
            Path::new(working_dir)
                .expand_home()
                .map(|expanded| {
                    absolute_path(expanded).map(|absolute| search_paths.push(absolute))
                })
                .ok();
        }
    }

    let package_info = PackageInfo {
        name: "Memospot".into(), // Same as productName from Tauri.toml. Will resolve to `/usr/lib/Memospot`.
        version: semver::Version::new(0, 0, 0),
        authors: "",
        description: "",
        crate_name: "",
    };
    let resource_dir = tauri_resource_dir(&package_info, &tauri::Env::default())
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    search_paths.extend([
        // Tauri uses `canonicalize()` to resolve the resource directory, which adds a `\\?\` prefix on Windows.
        resource_dir.trim_start_matches(r#"\\?\"#).into(),
        rtcfg.paths.memospot_data.clone(),
        rtcfg.paths.memospot_cwd.clone(),
    ]);

    debug!("looking for `dist` folder at {search_paths:#?}");
    for path in search_paths {
        if path.as_os_str().is_empty() {
            continue;
        }
        if path.join("dist").is_dir() {
            return path;
        }
    }
    // Fallback to data directory.
    rtcfg.paths.memospot_data.clone()
}

/// Make environment variable key suitable for Memos server.
fn make_env_key(key: &str) -> String {
    let uppercased_key = key.to_uppercase().replace("-", "_");
    if uppercased_key.starts_with("MEMOS_") {
        return uppercased_key;
    }
    format!("MEMOS_{uppercased_key}")
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

    // Add user-provided environment variables.
    if rtcfg.yaml.memos.env.enabled.unwrap_or_default() {
        if let Some(memos_env) = &rtcfg.yaml.memos.env.vars {
            for (key, value) in memos_env {
                env_vars.insert(key.into(), value.into());
            }
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
/// Supports:
///     - v0.23.0+ (/api/v1/workspace)
///     - v0.26.0+ (/api/v1/instance)
///
/// TODO: invert the endpoint priority when Memos v0.26.0 is out.
pub async fn query_version(memos_url: &str) -> Result<String, anyhow::Error> {
    const TIMEOUT_MS: u64 = 1_000;
    const ENDPOINTS: [&str; 2] = ["api/v1/workspace/profile", "api/v1/instance/profile"];

    let mut last_error = anyhow!("failed to query server version via API");

    for endpoint in ENDPOINTS {
        let endpoint = format!("{memos_url}{endpoint}");
        let url = match reqwest::Url::parse(&endpoint) {
            Ok(url) => url,
            Err(e) => {
                return Err(anyhow!("failed to parse server URL: {}", e));
            }
        };
        let client = reqwest::Client::new();
        let request = client
            .get(url)
            .header("User-Agent", "Memospot")
            .timeout(std::time::Duration::from_millis(TIMEOUT_MS))
            .send()
            .await;

        match request {
            Ok(response) => {
                if !response.status().is_success() {
                    last_error =
                        anyhow!("server responded with status code {}", response.status());
                    continue;
                }
                let json = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or_default();
                let version = json
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                return Ok(version.to_string());
            }
            Err(e) => {
                last_error = e.into();
                continue;
            }
        }
    }
    Err(last_error)
}

/// Poll Memos server until the API responds.
///
/// Server version is queried and stored in the global state, available via [`memos::VersionStore::get()`].
pub async fn wait_api_ready(memos_url: &str) {
    const INTERVAL_MS: u64 = 100;
    const TIMEOUT_MS: u128 = 15_000;

    let mut version = String::new();
    let mut last_error = String::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(INTERVAL_MS));

    let time_start = tokio::time::Instant::now();
    loop {
        if time_start.elapsed().as_millis() > TIMEOUT_MS {
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
            "failed to query server version via API: {last_error}. Giving up after {TIMEOUT_MS} ms."
        );
        return;
    }
    info!(
        "API ready in <{} ms. Version: {}.",
        time_start.elapsed().as_millis(),
        version
    );
    VersionStore::set(version);
}

/// Ping the Memos API to check if it is ready.
pub async fn ping_api(memos_url: &str, timeout_millis: u64) -> Result<bool, String> {
    let config = RuntimeConfig::from_global_store();
    let url = memos_url.trim_end_matches('/');
    let endpoint = format!("{url}/healthz");

    let url = reqwest::Url::parse(&endpoint).unwrap();
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(url)
        .header("User-Agent", &config.user_agent)
        .timeout(std::time::Duration::from_millis(if timeout_millis < 100 {
            1000
        } else {
            timeout_millis
        }))
        .send()
        .await
    {
        if response.status().is_success() {
            if let Ok(body) = response.text().await {
                if body.starts_with("Service ready.") {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_store() {
        assert_eq!(VersionStore::get(), "");

        // set version to 1.0.0 in the main thread
        VersionStore::set("1.0.0");
        assert_eq!(VersionStore::get(), "1.0.0");

        std::thread::spawn(|| {
            assert_eq!(VersionStore::get(), "1.0.0");
        })
        .join()
        .unwrap();

        tokio::spawn(async {
            assert_eq!(VersionStore::get(), "1.0.0");
        })
        .await
        .unwrap();

        // set version to 1.0.1 in the async runtime
        tokio::spawn(async {
            VersionStore::set("1.0.1");
            assert_eq!(VersionStore::get(), "1.0.1");
        })
        .await
        .unwrap();

        assert_eq!(VersionStore::get(), "1.0.1");

        std::thread::spawn(|| {
            assert_eq!(VersionStore::get(), "1.0.1");
        })
        .join()
        .unwrap();

        // set version to 1.0.2 in another thread
        std::thread::spawn(|| {
            VersionStore::set("1.0.2");
            assert_eq!(VersionStore::get(), "1.0.2");
        })
        .join()
        .unwrap();

        assert_eq!(VersionStore::get(), "1.0.2");

        tokio::spawn(async {
            assert_eq!(VersionStore::get(), "1.0.2");
        })
        .await
        .unwrap();
    }
}
