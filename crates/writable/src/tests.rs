use crate::PathExt;

#[cfg(not(target_os = "windows"))]
use nix::unistd::Uid;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Result, Write};
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_writable_directory() -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    assert!(&tmp_dir.path().to_path_buf().is_writable()); // PathBuf
    assert!(tmp_dir.path().is_writable()); // Path
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_unwritable_directory() -> Result<()> {
    if Uid::effective().is_root() {
        return Ok(());
    }

    let unwritable = PathBuf::from("/");
    assert!(!&unwritable.is_writable()); // PathBuf
    assert!(!unwritable.as_path().is_writable()); // Path

    Ok(())
}

#[test]
fn test_file() -> Result<()> {
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_file_path = temp_file.path();

    if let Ok(mut file) = File::create(temp_file_path) {
        let uuid = Uuid::new_v4();
        let test_content: &str = &uuid.to_string();
        println!("test_content: {}", test_content);

        file.write_all(test_content.as_bytes())?;
        file.flush()?;

        assert!(&temp_file_path.to_path_buf().is_writable()); // PathBuf
        assert!(&temp_file_path.is_writable()); // Path

        // make sure file content is unchanged after using is_writable() on it
        assert_eq!(fs::read_to_string(&temp_file)?, test_content);
        return Ok(());
    }

    Err(ErrorKind::Other.into())
}
