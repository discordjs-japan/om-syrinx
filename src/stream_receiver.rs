use napi::tokio::sync::{mpsc, oneshot};

use crate::error::{SyrinxError, SyrinxResult};

pub struct SyrinxStreamReceiver {
  construct: Option<oneshot::Receiver<SyrinxResult<()>>>,
  read: mpsc::Receiver<SyrinxResult<Vec<u8>>>,
}

impl SyrinxStreamReceiver {
  pub fn new(
    construct: oneshot::Receiver<SyrinxResult<()>>,
    read: mpsc::Receiver<SyrinxResult<Vec<u8>>>,
  ) -> Self {
    Self {
      construct: Some(construct),
      read,
    }
  }

  pub async fn construct(&mut self) -> SyrinxResult<()> {
    let construct = self
      .construct
      .take()
      .ok_or(SyrinxError::AlreadyConstructed)?;

    construct
      .await
      .map_err(|_| SyrinxError::PeerDropped("SynthesizeTask"))?
  }

  pub async fn read(&mut self) -> SyrinxResult<Option<Vec<u8>>> {
    self.read.recv().await.transpose()
  }
}
