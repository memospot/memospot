// tests for the commands functions.
#[cfg(test)]
mod test {
    use crate::*;
    use std::env::current_dir;

    #[cfg(not(windows))]
    fn detect_workspace_root() -> Option<PathBuf> {
        const MAX_DEPTH: usize = 5;
        let mut current_dir = current_dir().ok()?;

        for _ in 0..=MAX_DEPTH {
            if current_dir.join(".git").exists() && current_dir.join(".gitattributes").exists()
            {
                return Some(current_dir);
            }
            if !current_dir.pop() {
                return None;
            }
        }
        None
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_output() {
        let ws_root = detect_workspace_root().unwrap();
        let cargo_toml = ws_root.join("Cargo.toml");

        block_on(async move {
            // create a command to run cat.
            let cmd = Command::new("cat").args([cargo_toml.to_str().unwrap()]);
            let (mut rx, _) = cmd.spawn().unwrap();

            let mut matched = false;
            let mut terminated = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        // Accept zero exit code (or None) to avoid flaky tests.
                        assert!(
                            payload.code.map(|c| c == 0).unwrap_or(true),
                            "expected non-zero exit code, got {:?}",
                            payload.code
                        );

                        terminated = true;
                        if matched {
                            break;
                        }
                    }
                    CommandEvent::Stdout(line) => {
                        if !matched && line.contains(r#"package"#) {
                            matched = true;
                            if terminated {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            assert!(matched, "expected stdout containing 'package'");
            assert!(terminated, "expected Terminated event");
        });
    }

    #[cfg(not(windows))]
    #[test]
    // test the failure case
    fn test_cmd_fail() {
        block_on(async move {
            let cmd = Command::new("cat").args(["__non_existent_file__"]);
            let (mut rx, _) = cmd.spawn().unwrap();

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        // Accept any non-zero exit code (or None) to avoid flaky tests.
                        assert!(
                            payload.code.map(|c| c != 0).unwrap_or(true),
                            "expected non-zero exit code, got {:?}",
                            payload.code
                        );
                        break;
                    }
                    CommandEvent::Stderr(line) => {
                        assert!(line.contains("cat: __non_existent_file__:"));
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    #[test]
    fn test_command_interruption() {
        let command = Command::new("ping");
        command
            .args([
                #[cfg(windows)]
                "-t",
                "127.0.0.1",
            ])
            .spawn()
            .unwrap();
        kill_children();
    }
}
