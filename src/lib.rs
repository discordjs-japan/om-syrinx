#![deny(clippy::all)]

use htsengine::HTSEngine;
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
  htsengine: HTSEngine,
}

#[napi]
impl AltJTalk {
  #[napi(factory)]
  pub fn from_config(config: AltJTalkConfig) -> Result<Self, Error> {
    let mut htsengine = HTSEngine::new();
    htsengine
      .load(vec![config.model])
      .map_err(|err| Error::new(Status::InvalidArg, err))?;
    Ok(Self {
      jpreprocess: JPreprocess::from_config(JPreprocessConfig {
        dictionary: SystemDictionaryConfig::File(config.dictionary.into()),
        user_dictionary: None,
      })
      .map_err(|err| Error::new(Status::InvalidArg, err))?,
      htsengine,
    })
  }
  #[napi]
  pub fn synthesize(
    &mut self,
    input_text: String,
    option: SynthesisOption,
  ) -> Result<Int16Array, Error> {
    if input_text.is_empty() {
      return Ok(Int16Array::new(vec![]));
    }

    option.apply_to_engine(&mut self.htsengine);
    let labels = self
      .jpreprocess
      .extract_fullcontext(&input_text)
      .map_err(|err| Error::new(Status::Unknown, err))?;
    let audio: Vec<i16> = self
      .htsengine
      .synthesize(labels)
      .map_err(|err| Error::new(Status::Unknown, err))?
      .into_iter()
      .map(|d| {
        if d < (i16::MIN as f64) {
          i16::MIN
        } else if d > (i16::MAX as f64) {
          i16::MAX
        } else {
          d as i16
        }
      })
      .collect();
    Ok(Int16Array::new(audio))
  }
}
