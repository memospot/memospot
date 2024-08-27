//! Checks whether a path is writable.
//!
//! Example:
//!
//! ```
//! use std::path::PathBuf;
//! use writable::PathExt;
//!
//! let is_writable = PathBuf::from("./public_html").is_writable();
//! ```

use std::path::{Path, PathBuf};
#[cfg(test)]
mod tests;

/// `is_writable()` extension trait for `Path` and `PathBuf`.
pub trait PathExt {
    /// Check if a path is writable.
    ///
    /// Path types:
    ///     - File: tries to open it with write permissions.
    ///     - Directory: tries to create a temporary file inside it.
    fn is_writable(&self) -> bool;
}

impl PathExt for PathBuf {
    fn is_writable(&self) -> bool {
        if self.is_file() {
            for retry in 0..10 {
                if retry > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if let Ok(file) = std::fs::OpenOptions::new().write(true).open(self) {
                    drop(file);
                    return true;
                }
            }
            return false;
        }

        if self.is_dir() {
            let mut test_file = self.to_owned();
            test_file = test_file.join("write_test");

            let mut count = 0;
            while test_file.exists() {
                test_file.set_extension(count.to_string());
                count += 1;

                if count > 100 {
                    return false;
                }
            }

            for retry in 0..10 {
                if retry > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if let Ok(file) = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&test_file)
                {
                    drop(file);
                    if std::fs::remove_file(&test_file).is_err() {
                        continue;
                    }
                    return true;
                }
            }
        }
        false
    }
}

impl PathExt for Path {
    fn is_writable(&self) -> bool {
        self.to_path_buf().is_writable()
    }
}
