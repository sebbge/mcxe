mod download;
pub use download::*;

mod launch;
pub use launch::*;

mod operation;
pub use operation::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::sync::watch::{Receiver, Sender};

//- Public task handle
#[derive(Clone)]
pub struct TaskHandle<T> {
  pub id: String,
  pub rx: Receiver<T>,
  cancel: CancelFlag,
}
impl TaskHandle<download::Download> {
  pub fn cancel(&self) {
    self.cancel.cancel();
  }
}
impl TaskHandle<launch::Process> {
  pub fn kill(&self) {
    self.cancel.cancel();
  }
}
impl TaskHandle<operation::Operation> {
  pub fn cancel(&self) {
    self.cancel.cancel();
  }
}

//- Private task manager
#[derive(Clone)]
struct InnerTask<T> {
  state: Arc<RwLock<T>>,
  tx: Sender<T>,
  cancel: CancelFlag,
}

//- Task cancel flag
#[derive(Default, Clone)]
struct CancelFlag(Arc<AtomicBool>);
impl CancelFlag {
  fn cancel(&self) {
    self.0.store(true, Ordering::Relaxed);
  }

  fn is_cancelled(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}
