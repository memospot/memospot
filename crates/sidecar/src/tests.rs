// tests for the commands functions.
#[cfg(test)]
mod test {
    use crate::*;
    use std::env::current_dir;

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

        // create a command to run cat.
        let cmd = Command::new("cat").args([ws_root.join("Cargo.toml").to_str().unwrap()]);
        let (mut rx, _) = cmd.spawn().unwrap();

        block_on(async move {
            let mut matched = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        if !matched && line.contains(r#"package"#) {
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
        let cmd = Command::new("cat").args(["__non_existent_file__"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        block_on(async move {
            let mut terminated = false;
            let mut stderr_received = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(1));
                        terminated = true;
                    }
                    CommandEvent::Stderr(line) => {
                        assert!(line.contains("cat: __non_existent_file__:"));
                        stderr_received = true;
                    }
                    _ => {}
                }
            }
            assert!(terminated, "expected Terminated event");
            assert!(stderr_received, "expected Stderr event");
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
