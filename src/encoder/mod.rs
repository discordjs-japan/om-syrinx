use jbonsai::speech::SpeechGenerator;

pub trait Encoder: Send {
  fn generate(&self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>>;
}

impl Encoder for Box<dyn Encoder> {
  fn generate(&self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    self.as_ref().generate(generator)
  }
}

mod config;
mod opus;
mod pcm;

pub use config::*;
