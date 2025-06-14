use super::super::{Task, TaskManager};
use serde::Serialize;
use tokio::sync::watch::Sender;

//--- Public operation state
#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperationKind {
  #[default]
  Unknown,
  Repairing,
  // ??
}

#[derive(Default, Clone)]
pub struct Operation {
  pub id: String,
  pub kind: OperationKind,
  pub out: Option<String>,
}
impl Task<Operation> {
  fn set_out(&mut self, tx: &Sender<Self>, out: Option<String>) {
    self.payload.out = out;
    self.send_update(tx);
  }
}

//- Private operation manager
impl TaskManager<Operation> {
  pub async fn set_out(&self, out: Option<String>) {
    self.0.state.write().await.set_out(&self.0.tx, out);
  }
}
