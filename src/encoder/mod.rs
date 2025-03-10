use jbonsai::{engine::Condition, speech::SpeechGenerator};

mod config;
mod opus;
mod pcm;

pub use config::*;

use crate::error::SyrinxResult;

use self::{opus::OpusEncoder, pcm::PcmEncoder};

pub trait Encoder: Send {
  fn new(condition: &Condition, config: &EncoderConfig) -> SyrinxResult<Self>
  where
    Self: Sized;

  fn generate(&mut self, generator: &mut SpeechGenerator) -> SyrinxResult<Vec<u8>>;
}

macro_rules! encoder_new {
    (($condition:ident, $config:ident), $(($variant:ident, $encoder:ident)),* $(,)?) => {
      match $config.r#type {
        $(EncoderType::$variant => {
          let encoder = $encoder::new($condition, $config)?;
          Ok(Box::new(encoder))
        })*
      }
    };
}

impl Encoder for Box<dyn Encoder> {
  fn new(condition: &Condition, config: &EncoderConfig) -> SyrinxResult<Self> {
    encoder_new!((condition, config), (Opus, OpusEncoder), (Raw, PcmEncoder))
  }

  fn generate(&mut self, generator: &mut SpeechGenerator) -> SyrinxResult<Vec<u8>> {
    self.as_mut().generate(generator)
  }
}
