//! Project-specific build-time helpers for the Memospot workspace.

pub mod git;
pub mod memos;
pub mod memospot;
pub mod workspace;

pub use workspace::{find_target_dir, find_workspace_root, find_workspace_root_from};
