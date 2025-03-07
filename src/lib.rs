#![deny(clippy::all)]

use encoder::EncoderConfig;
use error::SyrinxResult;
use napi::{
  bindgen_prelude::{AsyncTask, Buffer, Null},
  tokio::sync::{mpsc, oneshot, Mutex},
  Env, Task,
};

use stream_receiver::SyrinxStreamReceiver;
use synthesis_option::SynthesisOption;
use worker::SyrinxWorker;

#[macro_use]
extern crate napi_derive;

#[napi]
pub const JPREPROCESS_VERSION: &str = env!("JPREPROCESS_VERSION");
#[napi]
pub const JBONSAI_VERSION: &str = env!("JBONSAI_VERSION");

mod encoder;
mod error;
mod stream_receiver;
mod synthesis_option;
mod worker;

/// Configuration for `Syrinx`.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct SyrinxConfig {
  /// Dictionary file path.
  pub dictionary: String,
  /// User dictionary file path.
  pub user_dictionary: Option<String>,
  /// Model file paths.
  pub models: Vec<String>,
  /// Encoder configuration.
  pub encoder: EncoderConfig,
}

pub struct FromConfigTask {
  config: SyrinxConfig,
}

#[napi]
impl Task for FromConfigTask {
  type Output = Syrinx;
  type JsValue = Syrinx;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(Syrinx(SyrinxWorker::from_config(&self.config)?))
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output)
  }
}

struct SynthesizeTask {
  worker: SyrinxWorker,
  input_text: String,
  option: SynthesisOption,
  construct: Option<oneshot::Sender<SyrinxResult<()>>>,
  read: mpsc::Sender<SyrinxResult<Vec<u8>>>,
}

#[napi]
impl Task for SynthesizeTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    // error is handled via construct
    let _ = self.worker.synthesize(
      &self.input_text,
      &self.option,
      &mut self.construct,
      &self.read,
    );
    Ok(())
  }

  fn resolve(&mut self, _: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(())
  }
}

// no doc comments because this will be wrapped in `index.js`/`index.d.ts`
#[napi]
pub struct Syrinx(SyrinxWorker);

#[napi]
impl Syrinx {
  #[napi]
  pub fn from_config(config: SyrinxConfig) -> napi::Result<Self> {
    Ok(Self(SyrinxWorker::from_config(&config)?))
  }

  #[napi]
  pub fn from_config_async(config: SyrinxConfig) -> AsyncTask<FromConfigTask> {
    AsyncTask::new(FromConfigTask { config })
  }

  #[napi]
  pub fn object_mode(&self) -> bool {
    self.0.object_mode()
  }

  #[napi]
  pub fn synthesize(
    &self,
    env: Env,
    input_text: String,
    option: SynthesisOption,
  ) -> napi::Result<SyrinxStream> {
    let worker = self.0.clone();
    let (construct_tx, construct_rx) = oneshot::channel();
    let (read_tx, read_rx) = mpsc::channel(256);

    env.spawn(SynthesizeTask {
      worker,
      input_text,
      option,
      construct: Some(construct_tx),
      read: read_tx,
    })?;

    Ok(SyrinxStream(Mutex::new(SyrinxStreamReceiver::new(
      construct_rx,
      read_rx,
    ))))
  }
}

#[napi]
pub struct SyrinxStream(Mutex<SyrinxStreamReceiver>);

#[napi]
impl SyrinxStream {
  #[napi]
  pub async fn construct(&self) -> napi::Result<Null> {
    self.0.lock().await.construct().await?;
    Ok(Null)
  }

  #[napi]
  pub async fn read(&self) -> napi::Result<Option<Buffer>> {
    let buf = self.0.lock().await.read().await?;
    Ok(buf.map(Buffer::from))
  }
}
