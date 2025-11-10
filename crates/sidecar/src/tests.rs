// tests for the commands functions.
#[cfg(test)]
mod test {
    use crate::*;

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_output() {
        // create a command to run cat.
        let cmd = Command::new("cat").args(["src/tests.rs"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        block_on(async move {
            let mut matched = false;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        if !matched
                            && line.contains(
                                r#"let cmd = Command::new("cat").args(["src/tests.rs"]);"#,
                            )
                        {
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
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(1));
                    }
                    CommandEvent::Stderr(line) => {
                        assert!(line.contains("cat: __non_existent_file__:"));
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
