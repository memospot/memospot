// tests for the commands functions.
#[cfg(test)]
mod test {
    use crate::*;
    use build_utils::find_workspace_root;

    const TIMEOUT_MS: u128 = 5_000;

    #[test]
    fn test_cmd_output() {
        let ws_root = find_workspace_root().expect("failed to find workspace root");
        let cargo_toml = ws_root.join("Cargo.toml");

        block_on(async move {
            let mut time_start = tokio::time::Instant::now();

            let mut terminated = false;
            let mut matched = false;

            'outer: for _ in 1..5 {
                #[cfg(windows)]
                let cmd = Command::new("powershell").args([
                    "-Command",
                    "Get-Content",
                    &cargo_toml.to_string_lossy(),
                ]);
                #[cfg(not(windows))]
                let cmd = Command::new("cat").args([cargo_toml.to_string_lossy()]);

                let (mut rx, _) = cmd.spawn().expect("failed to spawn command");
                while let Some(event) = rx.recv().await {
                    if time_start.elapsed().as_millis() > TIMEOUT_MS {
                        kill_children();
                        time_start = tokio::time::Instant::now();
                        continue 'outer;
                    }
                    match event {
                        CommandEvent::Terminated(payload) => {
                            terminated = payload.code.map(|c| c == 0).unwrap_or(true);
                            if matched {
                                break 'outer;
                            }
                        }
                        CommandEvent::Stdout(line) => {
                            if !matched && line.contains(r#"package"#) {
                                matched = true;
                                if terminated {
                                    break 'outer;
                                }
                            }
                        }
                        _ => {
                            eprintln!("unexpected event: {:?}", event);
                        }
                    }
                }
            }
            assert!(terminated, "expected Terminated event with zero exit code");
            assert!(matched, "expected stdout containing 'package'");
        });
    }

    #[test]
    fn test_cmd_fail() {
        block_on(async move {
            let mut time_start = tokio::time::Instant::now();
            let mut terminated = false;
            let mut matched = false;

            'outer: for _ in 1..5 {
                #[cfg(not(windows))]
                let cmd = Command::new("cat").args(["__non_existent_file__"]);

                #[cfg(windows)]
                let cmd = Command::new("powershell").args([
                    "-Command",
                    "Get-Content",
                    "__non_existent_file__",
                ]);

                let (mut rx, _) = cmd.spawn().unwrap();

                while let Some(event) = rx.recv().await {
                    if time_start.elapsed().as_millis() > TIMEOUT_MS {
                        kill_children();
                        time_start = tokio::time::Instant::now();
                        continue 'outer;
                    }
                    match event {
                        CommandEvent::Terminated(payload) => {
                            terminated = payload.code.map(|c| c != 0).unwrap_or(true);
                            if matched {
                                break 'outer;
                            }

                            break;
                        }
                        CommandEvent::Stderr(line) => {
                            if !matched && line.contains(r#"__non_existent_file__"#) {
                                matched = true;
                                if terminated {
                                    break 'outer;
                                }
                            }
                        }
                        _ => {
                            eprintln!("unexpected event: {:?}", event);
                        }
                    }
                }
            }
            assert!(
                terminated,
                "expected Terminated event with non-zero exit code"
            );
            assert!(
                matched,
                "expected stderr containing '__non_existent_file__'"
            );
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
