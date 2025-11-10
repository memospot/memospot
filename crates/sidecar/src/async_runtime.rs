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

pub fn block_on<F: Future>(task: F) -> F::Output {
    let rt = RUNTIME.get_or_init(default_runtime);
    rt.block_on(task)
}
