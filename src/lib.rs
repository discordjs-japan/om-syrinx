#![deny(clippy::all)]

use std::sync::Arc;

use encoder::{Encoder, EncoderConfig};
use jbonsai::{engine::Engine, speech::SpeechGenerator};
use jpreprocess::{
  DefaultFetcher, JPreprocess, JPreprocessConfig, SystemDictionaryConfig, UserDictionaryConfig,
};
use napi::{
  bindgen_prelude::AsyncTask,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Error, JsFunction, Status, Task,
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

#[napi]
impl AltJTalk {
  #[napi(factory)]
  pub fn from_config(config: AltJTalkConfig) -> napi::Result<Self> {
    Ok(Self(AltJtalkWorker::from_config(config)?))
  }

  #[napi(ts_return_type = "Promise<PreparedSynthesizer>")]
  pub fn prepare(&self, input_text: String, option: SynthesisOption) -> AsyncTask<PrepareTask> {
    let worker = self.0.clone();
    AsyncTask::new(PrepareTask {
      worker,
      input_text,
      option,
    })
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

pub struct PrepareTask {
  worker: AltJtalkWorker,
  input_text: String,
  option: SynthesisOption,
}

impl Task for PrepareTask {
  type Output = PreparedSynthesizer;
  type JsValue = PreparedSynthesizer;

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

    let generator = self
      .worker
      .jbonsai
      .generator(labels)
      .map_err(|err| Error::new(Status::InvalidArg, err))?;
    let encoder: Box<dyn Encoder> =
      Encoder::new(&self.worker.jbonsai.condition, &self.worker.encoder_config)?;

    Ok(PreparedSynthesizer {
      generator: Some(Box::new(generator)),
      encoder: Some(encoder),
    })
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi]
pub struct PreparedSynthesizer {
  generator: Option<Box<SpeechGenerator>>,
  encoder: Option<Box<dyn Encoder>>,
}

#[napi]
impl PreparedSynthesizer {
  #[napi(ts_return_type = "Promise<void>")]
  pub fn synthesize(
    &mut self,
    #[napi(
      ts_arg_type = "(...args: [err: null, frame: Buffer] | [err: Error, frame: null]) => void"
    )]
    push: JsFunction,
  ) -> napi::Result<AsyncTask<SyntheizeTask>> {
    let (Some(generator), Some(encoder)) = (self.generator.take(), self.encoder.take()) else {
      return Err(Error::new(
        Status::GenericFailure,
        "Synthesizer is already used".to_owned(),
      ));
    };

    let push = push.create_threadsafe_function(0, |ctx| {
      let buffer = ctx.env.create_buffer_with_data(ctx.value)?;
      Ok(vec![buffer.into_raw()])
    })?;

    Ok(AsyncTask::new(SyntheizeTask {
      generator,
      encoder,
      push,
    }))
  }
}

pub struct SyntheizeTask {
  generator: Box<SpeechGenerator>,
  encoder: Box<dyn Encoder>,
  push: ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>,
}

impl Task for SyntheizeTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    loop {
      let buf = self.encoder.generate(&mut self.generator)?;
      if buf.is_empty() {
        return Ok(());
      } else {
        self
          .push
          .call(Ok(buf), ThreadsafeFunctionCallMode::Blocking);
      }
    }
  }

  fn resolve(&mut self, _: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(())
  }

  fn finally(&mut self, env: Env) -> napi::Result<()> {
    self.push.unref(&env)
  }
}
