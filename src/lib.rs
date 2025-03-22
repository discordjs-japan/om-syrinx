#![deny(clippy::all)]

use std::sync::{Arc, Mutex};

use error::SyrinxError;
use napi::{
  Env, Task,
  bindgen_prelude::{AsyncTask, Buffer, Null},
};

use encoder::EncoderConfig;
use synthesis_option::SynthesisOption;
use synthesizer::SyrinxSynthesizer;
use worker::SyrinxWorker;

#[macro_use]
extern crate napi_derive;

#[napi]
pub const JPREPROCESS_VERSION: &str = env!("JPREPROCESS_VERSION");
#[napi]
pub const JBONSAI_VERSION: &str = env!("JBONSAI_VERSION");

mod encoder;
mod error;
mod synthesis_option;
mod synthesizer;
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
  pub fn stream(&self, input_text: String, option: SynthesisOption) -> SyrinxStream {
    let synthesizer = SyrinxSynthesizer::new(self.0.clone(), input_text, option);
    SyrinxStream {
      synthesizer: Arc::new(Mutex::new(synthesizer)),
      object_mode: self.0.object_mode(),
    }
  }
}

pub struct ConstructTask(Arc<Mutex<SyrinxSynthesizer>>);

#[napi]
impl Task for ConstructTask {
  type Output = ();
  type JsValue = Null;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let mut synthesizer = self.0.lock().map_err(|_| SyrinxError::LockFailed)?;
    synthesizer.initialize()?;
    Ok(())
  }

  fn resolve(&mut self, _: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(Null)
  }
}

pub struct ReadTask(Arc<Mutex<SyrinxSynthesizer>>);

#[napi]
impl Task for ReadTask {
  type Output = Vec<u8>;
  type JsValue = Option<Buffer>;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let mut synthesizer = self.0.lock().map_err(|_| SyrinxError::LockFailed)?;
    let buf = synthesizer.synthesize()?;
    Ok(buf)
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    if output.is_empty() {
      Ok(None)
    } else {
      Ok(Some(Buffer::from(output)))
    }
  }
}

// no doc comments because this will be wrapped in `index.js`/`index.d.ts`
#[napi]
pub struct SyrinxStream {
  synthesizer: Arc<Mutex<SyrinxSynthesizer>>,
  pub object_mode: bool,
}

#[napi]
impl SyrinxStream {
  #[napi]
  pub fn construct(&self) -> AsyncTask<ConstructTask> {
    AsyncTask::new(ConstructTask(self.synthesizer.clone()))
  }

  #[napi]
  pub fn read(&self) -> AsyncTask<ReadTask> {
    AsyncTask::new(ReadTask(self.synthesizer.clone()))
  }
}
