//! Handles spawning and killing child processes.
//!
//! Port of Tauri v1.8's `api::process::command`.
//! https://github.com/tauri-apps/tauri/blob/cc3d8e77313672f25520e278bbe8fae1b275a735/core/tauri/src/api/process/command.rs
//!

use anyhow::{format_err, Result};
pub use encoding_rs::Encoding;
use log::debug;
use os_pipe::{pipe, PipeReader, PipeWriter};
use serde::Serialize;
#[cfg(unix)]
use shared_child::unix::SharedChildExt;
use shared_child::SharedChild;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    collections::HashMap,
    io::{BufReader, Write},
    path::PathBuf,
    process::{Command as StdCommand, Stdio},
    sync::{Arc, Mutex, RwLock},
    thread::spawn,
};
use tauri::async_runtime::{block_on as block_on_task, channel, Receiver, Sender};
use tauri_utils::platform;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type ChildStore = Arc<Mutex<HashMap<u32, Arc<SharedChild>>>>;

fn commands() -> &'static ChildStore {
    use std::sync::LazyLock;
    static STORE: LazyLock<ChildStore> = LazyLock::new(Default::default);
    &STORE
}

/// Kills all child processes created with [`Command`].
///
/// Tries to send a SIGINT on UNIX systems.
///
/// By default, it's called before the [`crate::App`] exits.
pub fn kill_children() {
    let commands = commands().lock().unwrap();
    let children = commands.values();
    for child in children {
        debug!("sidecar: terminating pid {}", child.id());

        #[cfg(unix)]
        {
            const TIMEOUT_MS: u128 = 1_500;
            const SIGINT: i32 = 2;
            child.send_signal(SIGINT).ok();

            let time_start = std::time::Instant::now();
            let mut timed_out = false;
            while child.try_wait().is_ok_and(|x| x.is_none()) {
                if time_start.elapsed().as_millis() > TIMEOUT_MS {
                    timed_out = true;
                    break;
                };
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            if timed_out {
                debug!(
                    "sidecar: timed out ({} ms) waiting for pid {} to exit",
                    TIMEOUT_MS,
                    child.id()
                );
            } else {
                debug!(
                    "sidecar: pid {} exited gracefully in <{:?}ms",
                    child.id(),
                    time_start.elapsed().as_millis()
                );
            }
        }

        child.kill().ok();
    }
}

/// Payload for the [`CommandEvent::Terminated`] command event.
#[derive(Debug, Clone, Serialize)]
pub struct TerminatedPayload {
    /// Exit code of the process.
    pub code: Option<i32>,
    /// If the process was terminated by a signal, represents that signal.
    pub signal: Option<i32>,
}

/// AN event sent to the command callback.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "payload")]
#[non_exhaustive]
pub enum CommandEvent {
    /// Stderr bytes until a newline (\n) or carriage return (\r) is found.
    Stderr(String),
    /// Stdout bytes until a newline (\n) or carriage return (\r) is found.
    Stdout(String),
    /// An error happened waiting for the command to finish or converting the stdout/stderr bytes to a UTF-8 string.
    Error(String),
    /// Command process terminated.
    Terminated(TerminatedPayload),
}

/// The type to spawn commands.
#[derive(Debug)]
pub struct Command {
    program: String,
    args: Vec<String>,
    env_clear: bool,
    env: HashMap<String, String>,
    current_dir: Option<PathBuf>,
    encoding: Option<&'static Encoding>,
}

/// Spawned child process.
#[derive(Debug)]
pub struct CommandChild {
    inner: Arc<SharedChild>,
    stdin_writer: PipeWriter,
}

impl CommandChild {
    /// Writes to process stdin.
    pub fn write(&mut self, buf: &[u8]) -> tauri::Result<()> {
        self.stdin_writer.write_all(buf)?;
        Ok(())
    }

    /// Sends a kill signal to the child.
    pub fn kill(self) -> tauri::Result<()> {
        self.inner.kill()?;
        Ok(())
    }

    /// Returns the process pid.
    pub fn pid(&self) -> u32 {
        self.inner.id()
    }
}

/// Describes the result of a process after it has terminated.
#[derive(Debug)]
pub struct ExitStatus {
    code: Option<i32>,
}

impl ExitStatus {
    /// Returns the exit code of the process, if any.
    pub fn code(&self) -> Option<i32> {
        self.code
    }

    /// Returns true if exit status is zero. Signal termination is not considered a success, and success is defined as a zero exit status.
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

/// The output of a finished process.
#[derive(Debug)]
pub struct Output {
    /// The status (exit code) of the process.
    pub status: ExitStatus,
    /// The data that the process wrote to stdout.
    pub stdout: String,
    /// The data that the process wrote to stderr.
    pub stderr: String,
}

fn relative_command_path(command: String) -> Result<String> {
    match platform::current_exe()?.parent() {
        #[cfg(windows)]
        Some(exe_dir) => Ok(format!("{}\\{command}.exe", exe_dir.display())),
        #[cfg(not(windows))]
        Some(exe_dir) => Ok(format!("{}/{command}", exe_dir.display())),
        None => Err(format_err!("Could not evaluate executable dir")),
    }
}

impl From<Command> for StdCommand {
    fn from(cmd: Command) -> StdCommand {
        let mut command = StdCommand::new(cmd.program);
        command.args(cmd.args);
        command.stdout(Stdio::piped());
        command.stdin(Stdio::piped());
        command.stderr(Stdio::piped());
        if cmd.env_clear {
            command.env_clear();
        }
        command.envs(cmd.env);
        if let Some(current_dir) = cmd.current_dir {
            command.current_dir(current_dir);
        }
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        command
    }
}

impl Command {
    /// Creates a new Command for launching the given program.
    pub fn new<S: Into<String>>(program: S) -> Self {
        Self {
            program: program.into(),
            args: Default::default(),
            env_clear: false,
            env: Default::default(),
            current_dir: None,
            encoding: None,
        }
    }

    /// Creates a new Command for launching the given sidecar program.
    ///
    /// A sidecar program is an embedded external binary in order to make your application work
    /// or to prevent users having to install additional dependencies (e.g. Node.js, Python, etc.).
    pub fn new_sidecar<S: Into<String>>(program: S) -> Result<Self> {
        Ok(Self::new(relative_command_path(program.into())?))
    }

    /// Appends arguments to the command.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.args.push(arg.as_ref().to_string());
        }
        self
    }

    /// Clears the entire environment map for the child process.
    #[must_use]
    pub fn env_clear(mut self) -> Self {
        self.env_clear = true;
        self
    }

    /// Adds or updates multiple environment variable mappings.
    #[must_use]
    pub fn envs(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Sets the working directory for the child process.
    #[must_use]
    pub fn current_dir(mut self, current_dir: PathBuf) -> Self {
        self.current_dir.replace(current_dir);
        self
    }

    /// Sets the character encoding for stdout/stderr.
    #[must_use]
    pub fn encoding(mut self, encoding: &'static Encoding) -> Self {
        self.encoding.replace(encoding);
        self
    }

    /// Spawns the command.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use crate::memospot_lib::process::{Command, CommandEvent};
    /// tauri::async_runtime::spawn(async move {
    ///   let (mut rx, mut child) = Command::new("cargo")
    ///     .args(["tauri", "dev"])
    ///     .spawn()
    ///     .expect("Failed to spawn cargo");
    ///
    ///   let mut i = 0;
    ///   while let Some(event) = rx.recv().await {
    ///     if let CommandEvent::Stdout(line) = event {
    ///       println!("got: {}", line);
    ///       i += 1;
    ///       if i == 4 {
    ///         child.write("message from Rust\n".as_bytes()).unwrap();
    ///         i = 0;
    ///       }
    ///     }
    ///   }
    /// });
    /// ```
    pub fn spawn(self) -> tauri::Result<(Receiver<CommandEvent>, CommandChild)> {
        let encoding = self.encoding;
        let mut command: StdCommand = self.into();
        let (stdout_reader, stdout_writer) = pipe()?;
        let (stderr_reader, stderr_writer) = pipe()?;
        let (stdin_reader, stdin_writer) = pipe()?;
        command.stdout(stdout_writer);
        command.stderr(stderr_writer);
        command.stdin(stdin_reader);

        let shared_child = SharedChild::spawn(&mut command)?;
        let child = Arc::new(shared_child);
        let child_ = child.clone();
        let guard = Arc::new(RwLock::new(()));

        commands().lock().unwrap().insert(child.id(), child.clone());

        let (tx, rx) = channel(1);

        spawn_pipe_reader(
            tx.clone(),
            guard.clone(),
            stdout_reader,
            CommandEvent::Stdout,
            encoding,
        );
        spawn_pipe_reader(
            tx.clone(),
            guard.clone(),
            stderr_reader,
            CommandEvent::Stderr,
            encoding,
        );

        spawn(move || {
            let _ = match child_.wait() {
                Ok(status) => {
                    let _l = guard.write().unwrap();
                    commands().lock().unwrap().remove(&child_.id());
                    block_on_task(async move {
                        tx.send(CommandEvent::Terminated(TerminatedPayload {
                            code: status.code(),
                            #[cfg(windows)]
                            signal: None,
                            #[cfg(unix)]
                            signal: status.signal(),
                        }))
                        .await
                    })
                }
                Err(e) => {
                    let _l = guard.write().unwrap();
                    block_on_task(
                        async move { tx.send(CommandEvent::Error(e.to_string())).await },
                    )
                }
            };
        });

        Ok((
            rx,
            CommandChild {
                inner: child,
                stdin_writer,
            },
        ))
    }

    /// Executes a command as a child process, waiting for it to finish and collecting its exit status.
    /// Stdin, stdout and stderr are ignored.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use crate::memospot_lib::process::Command;
    /// let status = Command::new("which").args(["ls"]).status().unwrap();
    /// println!("`which` finished with status: {:?}", status.code());
    /// ```
    pub fn status(self) -> tauri::Result<ExitStatus> {
        let (mut rx, _child) = self.spawn()?;
        let code = block_on_task(async move {
            let mut code = None;
            #[allow(clippy::collapsible_match)]
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Terminated(payload) = event {
                    code = payload.code;
                }
            }
            code
        });
        Ok(ExitStatus { code })
    }

    /// Executes the command as a child process, waiting for it to finish and collecting all of its output.
    /// Stdin is ignored.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use crate::memospot_lib::process::{Command, CommandEvent};
    /// let output = Command::new("echo").args(["TAURI"]).output().unwrap();
    /// assert!(output.status.success());
    /// assert_eq!(output.stdout, "TAURI");
    /// ```
    pub fn output(self) -> tauri::Result<Output> {
        let (mut rx, _child) = self.spawn()?;

        let output = block_on_task(async move {
            let mut code = None;
            let mut stdout = String::new();
            let mut stderr = String::new();
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        code = payload.code;
                    }
                    CommandEvent::Stdout(line) => {
                        stdout.push_str(line.as_str());
                        stdout.push('\n');
                    }
                    CommandEvent::Stderr(line) => {
                        stderr.push_str(line.as_str());
                        stderr.push('\n');
                    }
                    CommandEvent::Error(_) => {}
                }
            }
            Output {
                status: ExitStatus { code },
                stdout,
                stderr,
            }
        });

        Ok(output)
    }
}

fn spawn_pipe_reader<F: Fn(String) -> CommandEvent + Send + Copy + 'static>(
    tx: Sender<CommandEvent>,
    guard: Arc<RwLock<()>>,
    pipe_reader: PipeReader,
    wrapper: F,
    character_encoding: Option<&'static Encoding>,
) {
    spawn(move || {
        let _lock = guard.read().unwrap();
        let mut reader = BufReader::new(pipe_reader);

        let mut buf = Vec::new();
        loop {
            buf.clear();
            match tauri_utils::io::read_line(&mut reader, &mut buf) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    let tx_ = tx.clone();
                    let line = match character_encoding {
                        Some(encoding) => Ok(encoding.decode_with_bom_removal(&buf).0.into()),
                        None => String::from_utf8(buf.clone()),
                    };
                    block_on_task(async move {
                        let _ = match line {
                            Ok(line) => tx_.send(wrapper(line)).await,
                            Err(e) => tx_.send(CommandEvent::Error(e.to_string())).await,
                        };
                    });
                }
                Err(e) => {
                    let tx_ = tx.clone();
                    let _ = block_on_task(async move {
                        tx_.send(CommandEvent::Error(e.to_string())).await
                    });
                    break;
                }
            }
        }
    });
}

// tests for the commands functions.
#[cfg(test)]
mod test {
    #[cfg(not(windows))]
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_output() {
        // create a command to run cat.
        let cmd = Command::new("cat").args(["src/main.rs"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        block_on_task(async move {
            let mut matched = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        if !matched && line.contains(r#"#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]"#) {
                            matched = true;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            assert!(matched);
        });
    }

    #[cfg(not(windows))]
    #[test]
    // test the failure case
    fn test_cmd_fail() {
        let cmd = Command::new("cat").args(["src"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        block_on_task(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(1));
                    }
                    CommandEvent::Stderr(line) => {
                        assert!(line.contains("cat: src:"));
                    }
                    _ => {}
                }
            }
        });
    }
}
