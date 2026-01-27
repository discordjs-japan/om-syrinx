use std::sync::Arc;

use jbonsai::{engine::Engine, speech::SpeechGenerator};
use jpreprocess::{DefaultTokenizer, JPreprocess, SystemDictionaryConfig, error::JPreprocessError};
use lindera_dictionary::loader::user_dictionary::UserDictionaryLoader;

use crate::{
  SyrinxConfig,
  encoder::{Encoder, EncoderConfig},
  error::SyrinxResult,
  synthesis_option::SynthesisOption,
};

#[derive(Clone)]
pub struct SyrinxWorker {
  jpreprocess: Arc<JPreprocess<DefaultTokenizer>>,
  jbonsai: Engine,
  encoder_config: EncoderConfig,
}

impl SyrinxWorker {
  pub fn from_config(config: &SyrinxConfig) -> SyrinxResult<Self> {
    let system_dictionary =
      SystemDictionaryConfig::File(config.dictionary.clone().into()).load()?;
    let user_dictionary = config
      .user_dictionary
      .as_ref()
      .map(|path| UserDictionaryLoader::load_from_bin(path).map_err(JPreprocessError::LinderaError))
      .transpose()?;

    let jpreprocess = JPreprocess::with_dictionaries(system_dictionary, user_dictionary);
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

  pub fn prepare(
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
}
