#![deny(clippy::all)]

use htsengine::HTSEngine;
use jpreprocess::{JPreprocess, JPreprocessConfig, SystemDictionaryConfig};
use napi::{Error, Status};

#[macro_use]
extern crate napi_derive;

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
  pub fn synthesize(&mut self, input_text: String) -> Result<Vec<i16>, Error> {
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
    Ok(audio)
  }
}
