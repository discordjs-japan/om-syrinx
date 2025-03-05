#![deny(clippy::all)]

use std::sync::Arc;

use encoder::{Encoder, EncoderConfig};
use error::SyrinxResult;
use jbonsai::{engine::Engine, speech::SpeechGenerator};
use jpreprocess::{DefaultTokenizer, JPreprocess, JPreprocessConfig, SystemDictionaryConfig};
use napi::{
  bindgen_prelude::AsyncTask,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Error, JsFunction, Status, Task,
};
use synthesis_option::SynthesisOption;

#[macro_use]
extern crate napi_derive;

#[napi]
pub const JPREPROCESS_VERSION: &str = env!("JPREPROCESS_VERSION");
#[napi]
pub const JBONSAI_VERSION: &str = env!("JBONSAI_VERSION");

mod encoder;
mod error;
mod synthesis_option;

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

// no doc comments because this will be wrapped in `index.js`/`index.d.ts`
#[napi]
pub struct Syrinx(SyrinxWorker);

#[napi]
impl Syrinx {
  #[napi(factory)]
  pub fn from_config(config: SyrinxConfig) -> napi::Result<Self> {
    Ok(Self(SyrinxWorker::from_config(&config)?))
  }

  #[napi(ts_return_type = "Promise<Syrinx>")]
  pub fn from_config_async(config: SyrinxConfig) -> AsyncTask<FromConfigTask> {
    AsyncTask::new(FromConfigTask { config })
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
struct SyrinxWorker {
  jpreprocess: Arc<JPreprocess<DefaultTokenizer>>,
  jbonsai: Engine,
  encoder_config: EncoderConfig,
}

impl SyrinxWorker {
  fn from_config(config: &SyrinxConfig) -> SyrinxResult<Self> {
    let jpreprocess = JPreprocess::from_config(JPreprocessConfig {
      dictionary: SystemDictionaryConfig::File(config.dictionary.clone().into()),
      user_dictionary: config
        .user_dictionary
        .as_ref()
        .map(|path| serde_json::json!({ "path": path })),
    })?;
    let jbonsai = Engine::load(&config.models)?;

    Ok(Self {
      jpreprocess: Arc::new(jpreprocess),
      jbonsai,
      encoder_config: config.encoder.clone(),
    })
  }
}

pub struct FromConfigTask {
  config: SyrinxConfig,
}

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

pub struct PrepareTask {
  worker: SyrinxWorker,
  input_text: String,
  option: SynthesisOption,
}

impl PrepareTask {
  fn prepare(&mut self) -> SyrinxResult<PreparedSynthesizer> {
    self
      .option
      .apply_to_engine(&mut self.worker.jbonsai.condition)?;

    let labels = self
      .worker
      .jpreprocess
      .extract_fullcontext(&self.input_text)?;

    let generator = self.worker.jbonsai.generator(labels)?;
    let encoder: Box<dyn Encoder> =
      Encoder::new(&self.worker.jbonsai.condition, &self.worker.encoder_config)?;

    Ok(PreparedSynthesizer {
      generator: Some(Box::new(generator)),
      encoder: Some(encoder),
    })
  }
}

impl Task for PrepareTask {
  type Output = PreparedSynthesizer;
  type JsValue = PreparedSynthesizer;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(self.prepare()?)
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
