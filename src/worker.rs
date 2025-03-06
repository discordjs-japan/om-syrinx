use std::sync::Arc;

use jbonsai::{engine::Engine, speech::SpeechGenerator};
use jpreprocess::{DefaultTokenizer, JPreprocess, JPreprocessConfig, SystemDictionaryConfig};
use napi::tokio::sync::{mpsc, oneshot};

use crate::{
  encoder::{Encoder, EncoderConfig},
  error::{SyrinxError, SyrinxResult},
  synthesis_option::SynthesisOption,
  SyrinxConfig,
};

#[derive(Clone)]
pub struct SyrinxWorker {
  jpreprocess: Arc<JPreprocess<DefaultTokenizer>>,
  jbonsai: Engine,
  encoder_config: EncoderConfig,
}

impl SyrinxWorker {
  pub fn from_config(config: &SyrinxConfig) -> SyrinxResult<Self> {
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

  pub fn object_mode(&self) -> bool {
    self.encoder_config.r#type.object_mode()
  }

  fn construct(
    &mut self,
    input_text: &str,
    option: &SynthesisOption,
  ) -> SyrinxResult<(SpeechGenerator, Box<dyn Encoder>)> {
    option.apply_to_engine(&mut self.jbonsai.condition)?;

    let labels = self.jpreprocess.extract_fullcontext(input_text)?;

    let generator = self.jbonsai.generator(labels)?;
    let encoder = Encoder::new(&self.jbonsai.condition, &self.encoder_config)?;

    Ok((generator, encoder))
  }

  pub fn synthesize(
    &mut self,
    input_text: &str,
    option: &SynthesisOption,
    construct: &mut Option<oneshot::Sender<SyrinxResult<()>>>,
    read: &mpsc::Sender<SyrinxResult<Vec<u8>>>,
  ) -> SyrinxResult<()> {
    let construct = construct.take().ok_or(SyrinxError::AlreadyConstructed)?;
    let (mut generator, mut encoder) = match self.construct(input_text, option) {
      Ok(ret) => {
        construct
          .send(Ok(()))
          .map_err(|_| SyrinxError::PeerDropped("SyrinxStreamReceiver"))?;
        ret
      }
      Err(e) => {
        construct
          .send(Err(e))
          .map_err(|_| SyrinxError::PeerDropped("SyrinxStreamReceiver"))?;
        return Err(SyrinxError::TaskEnded);
      }
    };

    loop {
      match encoder.generate(&mut generator) {
        Ok(buf) if buf.is_empty() => return Ok(()),
        Ok(buf) => read
          .blocking_send(Ok(buf))
          .map_err(|_| SyrinxError::PeerDropped("SyrinxStreamReceiver"))?,
        Err(e) => {
          read
            .blocking_send(Err(e))
            .map_err(|_| SyrinxError::PeerDropped("SyrinxStreamReceiver"))?;
          return Err(SyrinxError::TaskEnded);
        }
      }
    }
  }
}
