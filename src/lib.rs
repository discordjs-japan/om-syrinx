#![deny(clippy::all)]

use std::sync::Arc;

use encoder::{Encoder, EncoderConfig};
use jbonsai::engine::Engine;
use jpreprocess::{
  DefaultFetcher, JPreprocess, JPreprocessConfig, SystemDictionaryConfig, UserDictionaryConfig,
};
use napi::{
  bindgen_prelude::AsyncTask,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Error, JsFunction, JsUndefined, Status, Task,
};
use synthesis_option::SynthesisOption;

#[macro_use]
extern crate napi_derive;

mod encoder;
mod synthesis_option;

/// Configuration for `AltJTalk`.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct AltJTalkConfig {
  /// Dictionary file path.
  pub dictionary: String,
  /// User dictionary file path.
  pub user_dictionary: Option<String>,
  /// Model file paths.
  pub models: Vec<String>,
  /// Encoder configuration.
  pub encoder: EncoderConfig,
}

// no doc comments because this will be wrapped in `index.js`/`index.d.ts`
#[napi]
pub struct AltJTalk(AltJtalkWorker);

// separate `impl` block because rust-analyzer fails to expand `#[napi]` on `impl` block with `#[napi(factory)]`
#[napi]
impl AltJTalk {
  #[napi(factory)]
  pub fn from_config(config: AltJTalkConfig) -> napi::Result<Self> {
    Ok(Self(AltJtalkWorker::from_config(config)?))
  }
}

#[napi]
impl AltJTalk {
  #[napi(
    ts_args_type = "inputText: string, option: SynthesisOption, push: (...args: [err: null, frame: Buffer] | [err: Error, frame: null]) => void"
  )]
  pub fn synthesize(
    &mut self,
    input_text: String,
    option: SynthesisOption,
    push: JsFunction,
  ) -> napi::Result<AsyncTask<SynthesizeTask>> {
    let worker = self.0.clone();

    let push = push.create_threadsafe_function(0, |ctx| {
      let buffer = ctx.env.create_buffer_with_data(ctx.value)?;
      Ok(vec![buffer.into_raw()])
    })?;

    Ok(AsyncTask::new(SynthesizeTask {
      worker,
      input_text,
      option,
      push,
    }))
  }
}

#[derive(Clone)]
struct AltJtalkWorker {
  jpreprocess: Arc<JPreprocess<DefaultFetcher>>,
  jbonsai: Engine,
  encoder_config: EncoderConfig,
}

impl AltJtalkWorker {
  fn from_config(config: AltJTalkConfig) -> napi::Result<Self> {
    let jpreprocess = JPreprocess::from_config(JPreprocessConfig {
      dictionary: SystemDictionaryConfig::File(config.dictionary.into()),
      user_dictionary: config
        .user_dictionary
        .as_ref()
        .map(|path| UserDictionaryConfig {
          path: path.into(),
          kind: None,
        }),
    })
    .map_err(|err| Error::new(Status::InvalidArg, err))?;
    let jbonsai =
      Engine::load(&config.models).map_err(|err| Error::new(Status::InvalidArg, err))?;

    Ok(Self {
      jpreprocess: Arc::new(jpreprocess),
      jbonsai,
      encoder_config: config.encoder,
    })
  }
}

pub struct SynthesizeTask {
  worker: AltJtalkWorker,
  input_text: String,
  option: SynthesisOption,
  push: ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>,
}

impl Task for SynthesizeTask {
  type Output = ();
  type JsValue = JsUndefined;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    self
      .option
      .apply_to_engine(&mut self.worker.jbonsai.condition)
      .map_err(|err| Error::new(Status::InvalidArg, err))?;

    let labels = self
      .worker
      .jpreprocess
      .extract_fullcontext(&self.input_text)
      .map_err(|err| Error::new(Status::InvalidArg, err))?;
    if labels.len() <= 2 {
      return Ok(());
    }

    let mut generator = self
      .worker
      .jbonsai
      .generator(labels)
      .map_err(|err| Error::new(Status::InvalidArg, err))?;
    let mut encoder: Box<dyn Encoder> =
      Encoder::new(&self.worker.jbonsai.condition, &self.worker.encoder_config)?;

    loop {
      let buf = encoder.generate(&mut generator)?;
      if buf.is_empty() {
        return Ok(());
      } else {
        self
          .push
          .call(Ok(buf), ThreadsafeFunctionCallMode::Blocking);
      }
    }
  }

  fn resolve(&mut self, env: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    env.get_undefined()
  }
}
