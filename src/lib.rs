#![deny(clippy::all)]

use jbonsai::engine::Engine;
use jpreprocess::{JPreprocess, JPreprocessConfig, SystemDictionaryConfig};
use napi::{bindgen_prelude::Int16Array, Error, Status};
use synthesis_option::SynthesisOption;

#[macro_use]
extern crate napi_derive;

mod synthesis_option;

#[napi(object)]
pub struct AltJTalkConfig {
  pub dictionary: String,
  pub model: String,
}

#[napi]
pub struct AltJTalk {
  jpreprocess: JPreprocess,
  jbonsai: Engine,

  default_options: SynthesisOption,
}

#[napi]
impl AltJTalk {
  #[napi(factory)]
  pub fn from_config(config: AltJTalkConfig) -> Result<Self, Error> {
    let jbonsai = Engine::load(&[config.model]);

    let default_options = SynthesisOption::from_engine(&jbonsai.condition);

    Ok(Self {
      jpreprocess: JPreprocess::from_config(JPreprocessConfig {
        dictionary: SystemDictionaryConfig::File(config.dictionary.into()),
        user_dictionary: None,
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
    option.apply_to_engine(&mut self.jbonsai.condition, &self.default_options);
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
