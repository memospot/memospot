// tests for the commands functions.
#[cfg(test)]
mod test {
    use crate::*;
    use build_utils::find_workspace_root;

    #[test]
    fn test_cmd_output() {
        let ws_root = find_workspace_root().unwrap();
        let cargo_toml = ws_root.join("Cargo.toml");

        block_on(async move {
            #[cfg(windows)]
            let cmd = Command::new("powershell").args([
                "-Command",
                "Get-Content",
                cargo_toml.to_str().unwrap(),
            ]);

            #[cfg(not(windows))]
            let cmd = Command::new("cat").args([cargo_toml.to_str().unwrap()]);

            let (mut rx, _) = cmd.spawn().unwrap();

            let mut matched = false;
            let mut terminated = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
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

    #[test]
    fn test_cmd_fail() {
        block_on(async move {
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
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert!(
                            payload.code.map(|c| c != 0).unwrap_or(true),
                            "expected non-zero exit code, got {:?}",
                            payload.code
                        );
                        break;
                    }
                    CommandEvent::Stderr(line) => {
                        assert!(line.contains("__non_existent_file__"));
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
