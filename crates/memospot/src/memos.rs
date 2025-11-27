use crate::memos_version::MemosVersionStore;
use crate::utils::absolute_path;
use crate::{fl, memos_log};
use crate::{sqlite, RuntimeConfig};
use anyhow::{anyhow, Result};
use dialog::{error_dialog, panic_dialog};
use homedir::HomeDirExt;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, System};
use tauri::async_runtime;
use tauri::utils::platform::resource_dir as tauri_resource_dir;
use tauri_plugin_http::reqwest;
use tauri_utils::PackageInfo;
use tokio::io::AsyncWriteExt;

/// Cleanup orphaned Memos processes.
///
/// NOTE: there's a serious bug that prevents the `ExitRequested` event from
/// running on macOS when the app is closed via the dock context menu.
/// See: <https://github.com/tauri-apps/tauri/issues/9198>
///
/// This causes Memos to stay as an orphaned process, and it also makes the UI hang while
/// Memospot tries to checkpoint a database locked by a process it no longer controls.
///
/// This function is cross-platform and should help to recover from such situations.
pub fn find_and_kill_orphaned(rtcfg: &RuntimeConfig) {
    if let Some(pid) = get_last_pid(rtcfg) {
        debug!("unclean shutdown detected");
        let prev_port = rtcfg.__yaml__.memos.port.unwrap_or_default();
        let prev_url = format!("http://localhost:{}/", prev_port);

        async_runtime::block_on(async {
            if Ok(true) == ping_api(&prev_url, 2_000).await {
                warn!("detected orphaned Memos server (PID: {pid}). Attempting to terminate…");
                kill_pid(pid).await;

                // Give some time for the port to be released.
                std::thread::sleep(std::time::Duration::from_secs(2));
                return;
            }
            debug!("got no response from {prev_url}. No action taken.");
        });
    }
}

/// Get the last known Memos PID.
///
/// Should return None if it had a graceful shutdown.
pub fn get_last_pid(rtcfg: &RuntimeConfig) -> Option<u32> {
    let pid_file = rtcfg.paths.memospot_data.join("memos.pid");
    if pid_file.is_file() {
        return match fs::read_to_string(&pid_file) {
            Ok(content) => match content.trim().parse::<u32>() {
                Ok(pid) => Some(pid),
                Err(e) => {
                    warn!("Failed to parse PID from file {pid_file:#?}: {e}");
                    None
                }
            },
            Err(e) => {
                debug!("Failed to read PID file {pid_file:#?}: {e}");
                None
            }
        };
    }
    debug!("PID file does not exist: {pid_file:#?}");
    None
}

/// Save PID.
///
/// Stores the PID so we can use this to track and recover from ungraceful shutdowns.
///
/// Only a single PID is ever stored.
fn save_pid_file(pid: u32, file_path: &Path) {
    if file_path.is_dir() {
        panic_dialog("provided file path is a directory");
    }

    let pid_file = file_path.to_path_buf();
    let file_contents = pid.to_string();
    let time_start = tokio::time::Instant::now();

    async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        let mut last_error: String = "".into();
        loop {
            interval.tick().await;
            if time_start.elapsed() > tokio::time::Duration::from_secs(5) {
                warn!("timed out after 5 seconds while trying to write pid file");
                break;
            }

            let mut file = match tokio::fs::File::create(&pid_file).await {
                Ok(f) => f,
                Err(e) => {
                    let error = e.to_string();
                    if last_error != error {
                        warn!("unable to create pid file: {error}");
                        last_error = error;
                    }
                    continue;
                }
            };

            if let Err(e) = file.write_all(file_contents.as_bytes()).await {
                let error = e.to_string();
                if last_error != error {
                    warn!("unable to write pid file: {error}");
                    last_error = error;
                }
                continue;
            }

            if let Err(e) = file.flush().await {
                let error = e.to_string();
                if last_error != error {
                    warn!("unable to flush pid file: {error}");
                    last_error = error;
                }
                continue;
            }

            break;
        }
    });
}

fn remove_pid_file(rtcfg: &RuntimeConfig) {
    let pid_file = rtcfg.paths.memospot_data.join("memos.pid");
    let time_start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(5);

    async_runtime::block_on(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        let mut last_error: String = "".into();
        loop {
            interval.tick().await;
            if time_start.elapsed() > timeout {
                warn!("timed out after 5 seconds while trying to remove pid file");
                break;
            }

            if let Err(e) = tokio::fs::remove_file(&pid_file).await {
                let error = e.to_string();
                if last_error != error {
                    warn!("unable to remove pid file: {error}");
                    last_error = error;
                }
                continue;
            }
            debug!("pid file removed");
            break;
        }
    });
}

async fn kill_pid(pid: u32) {
    let time_start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(5);
    let sys = System::new_all();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    loop {
        interval.tick().await;
        if time_start.elapsed() > timeout {
            warn!("timed out after {timeout:?} seconds");
            break;
        }

        if let Some(process) = sys.process(Pid::from(pid as usize)) {
            if let Err(e) = process.kill_and_wait() {
                warn!("unable to kill pid {pid}: {e:?}")
            } else {
                let elapsed = time_start.elapsed().as_millis();
                warn!("killed pid {pid} after {elapsed} ms");
                break;
            }
        }
    }
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

    const MAX_RETRIES: usize = 2;
    let mut last_error = anyhow!("failed to spawn Memos server");

    for _ in 1..=MAX_RETRIES {
        let res = sidecar::Command::new(&command)
            .envs(env_vars.clone())
            .current_dir(cwd.clone())
            .spawn();

        match res {
            Ok(receiver, ..) => {
                if rtcfg.yaml.memospot.log.enabled.unwrap_or(false) {
                    let (rx_, _) = receiver;
                    async_runtime::spawn(async move {
                        memos_log::log_events(rx_).await;
                    });
                }

                let (_, child) = receiver;
                let pid_file = rtcfg.paths.memospot_data.join("memos.pid");

                save_pid_file(child.pid(), &pid_file);

                return Ok(());
            }
            Err(e) => {
                last_error = last_error.context(e);
                warn!("{last_error}");

                #[cfg(unix)]
                {
                    warn!("attempting to add executable permissions to the binary…");
                    if let Err(e) = (|| -> Result<(), anyhow::Error> {
                        let mut perms = fs::metadata(&command)?.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&command, perms)?;
                        Ok(())
                    })() {
                        let perm_err = anyhow!("failed to set permissions").context(e);
                        last_error = last_error.context(perm_err);
                        warn!("{last_error}");
                    } else {
                        info!("permissions added successfully. Attempting to relaunch…");
                    }
                }
                continue;
            }
        };
    }

    warn!("exceeded launch retries limit ({MAX_RETRIES}). Giving up.");
    Err(last_error)
}

/// Shutdown the Memos server and checkpoint the database.
pub fn shutdown() {
    let config = RuntimeConfig::from_global_store();
    if !config.is_managed_server {
        debug!("server is not managed by Memospot. No need to cleanup before exit");
        return;
    }

    debug!("shutting down server…");
    sidecar::kill_children();

    let db_file = config.paths.memos_db_file.clone();
    async_runtime::block_on(async move {
        sqlite::wait_checkpoint(&db_file).await;
    });
    remove_pid_file(&config);
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

/// Memos URL.
///
/// It's ensured to end with a slash.
///
/// If remote server is enabled, return the configured URL.
/// Otherwise, return the default Memos address for the spawned server.
pub fn get_url(rtcfg: &RuntimeConfig) -> String {
    let remote = &rtcfg.yaml.memospot.remote;
    let url = remote.url.as_deref().unwrap_or_default();

    if remote.enabled != Some(true) || url.is_empty() {
        return format!(
            "http://localhost:{}/",
            rtcfg.yaml.memos.port.unwrap_or_default()
        );
    }

    if !url.starts_with("http") {
        error_dialog!(fl!("error-invalid-server-url", url = url));
    }

    format!("{}/", url.trim_end_matches('/'))
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
                    let code = response.status();
                    last_error = anyhow!("server responded with status code {code}");
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
    MemosVersionStore::set(version);
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
