use std::future::Future;
use std::sync::OnceLock;
pub use tokio::{
    runtime::Runtime,
    sync::mpsc::{channel, Receiver, Sender},
};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn default_runtime() -> Runtime {
    Runtime::new().expect("failed to create Tokio runtime")
}

/// Runs a future to completion on the Tokio runtime. This is the runtime’s entry point.
///
/// This runs the given future on the current thread, blocking until it is complete, and yielding its resolved result. Any tasks or timers which the future spawns internally will be executed on the runtime.
/// See [`Runtime::block_on`].
pub fn block_on<F: Future>(task: F) -> F::Output {
    let rt = RUNTIME.get_or_init(default_runtime);
    rt.block_on(task)
}
