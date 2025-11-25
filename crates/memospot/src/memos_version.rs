use std::sync::LazyLock;
use std::sync::{Arc, Mutex};

use dialog::ExpectDialogExt;
#[derive(Clone, Debug, Default)]
pub struct MemosVersionStore {
    version: Arc<Mutex<String>>,
}
impl MemosVersionStore {
    /// Returns a reference to the global singleton instance of `VersionStore`.
    fn instance() -> &'static Self {
        static INSTANCE: LazyLock<MemosVersionStore> = LazyLock::new(Default::default);
        &INSTANCE
    }
    /// Get version previously stored by [`memos::wait_api_ready()`].
    pub fn get() -> String {
        MemosVersionStore::instance()
            .version
            .lock()
            .expect_dialog("unable to lock version store")
            .clone()
    }
    pub fn set(version: impl Into<String>) {
        let mut store = MemosVersionStore::instance()
            .version
            .lock()
            .expect_dialog("unable to lock version store");
        *store = version.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_store() {
        assert_eq!(MemosVersionStore::get(), "");

        // set version to 1.0.0 in the main thread
        MemosVersionStore::set("1.0.0");
        assert_eq!(MemosVersionStore::get(), "1.0.0");

        std::thread::spawn(|| {
            assert_eq!(MemosVersionStore::get(), "1.0.0");
        })
        .join()
        .unwrap();

        tokio::spawn(async {
            assert_eq!(MemosVersionStore::get(), "1.0.0");
        })
        .await
        .unwrap();

        // set version to 1.0.1 in the async runtime
        tokio::spawn(async {
            MemosVersionStore::set("1.0.1");
            assert_eq!(MemosVersionStore::get(), "1.0.1");
        })
        .await
        .unwrap();

        assert_eq!(MemosVersionStore::get(), "1.0.1");

        std::thread::spawn(|| {
            assert_eq!(MemosVersionStore::get(), "1.0.1");
        })
        .join()
        .unwrap();

        // set version to 1.0.2 in another thread
        std::thread::spawn(|| {
            MemosVersionStore::set("1.0.2");
            assert_eq!(MemosVersionStore::get(), "1.0.2");
        })
        .join()
        .unwrap();

        assert_eq!(MemosVersionStore::get(), "1.0.2");

        tokio::spawn(async {
            assert_eq!(MemosVersionStore::get(), "1.0.2");
        })
        .await
        .unwrap();
    }
}
