#![deny(clippy::all)]

use jbonsai::engine::{Condition, Engine};
use jpreprocess::{JPreprocess, JPreprocessConfig, SystemDictionaryConfig};
use lindera_dictionary::UserDictionaryConfig;
use napi::{bindgen_prelude::Int16Array, Error, Status};
use synthesis_option::SynthesisOption;

#[macro_use]
extern crate napi_derive;

mod synthesis_option;

#[napi(object)]
pub struct AltJTalkConfig {
  pub dictionary: String,
  pub user_dictionary: Option<String>,
  pub models: Vec<String>,
}

#[napi]
pub struct AltJTalk {
  jpreprocess: JPreprocess,
  jbonsai: Engine,

  default_options: Condition,
}

#[napi]
impl AltJTalk {
  #[napi(factory)]
  pub fn from_config(config: AltJTalkConfig) -> Result<Self, Error> {
    let jbonsai = Engine::load(&config.models);

    let default_options = jbonsai.condition.clone();

    Ok(Self {
      jpreprocess: JPreprocess::from_config(JPreprocessConfig {
        dictionary: SystemDictionaryConfig::File(config.dictionary.into()),
        user_dictionary: config
          .user_dictionary
          .as_ref()
          .map(|path| UserDictionaryConfig {
            path: path.into(),
            kind: None,
          }),
      })
      .map_err(|err| Error::new(Status::InvalidArg, err))?,
      jbonsai,
      default_options,
    })
  }
  #[napi]
  pub fn synthesize(
    &mut self,
    input_text: String,
    option: SynthesisOption,
  ) -> Result<Int16Array, Error> {
    self.jbonsai.condition = self.default_options.clone();
    option.apply_to_engine(&mut self.jbonsai.condition);

    let labels = self
      .jpreprocess
      .extract_fullcontext(&input_text)
      .map_err(|err| Error::new(Status::Unknown, err))?;
    if labels.len() <= 2 {
      return Ok(Int16Array::new(vec![]));
    }

    self.jbonsai.synthesize_from_strings(&labels);
    let audio: Vec<i16> = self
      .jbonsai
      .get_generated_speech()
      .iter()
      .map(|d| {
        if *d < (i16::MIN as f64) {
          i16::MIN
        } else if *d > (i16::MAX as f64) {
          i16::MAX
        } else {
          *d as i16
        }
      })
      .collect();
    Ok(Int16Array::new(audio))
  }
}
