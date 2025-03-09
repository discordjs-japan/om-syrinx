use jbonsai::speech::SpeechGenerator;

use crate::{
  encoder::Encoder, error::SyrinxResult, synthesis_option::SynthesisOption, worker::SyrinxWorker,
};

pub enum SyrinxSynthesizer {
  Uninitialized(SyrinxWorker, String, SynthesisOption),
  Initialized(SpeechGenerator, Box<dyn Encoder>),
}

impl SyrinxSynthesizer {
  pub fn new(worker: SyrinxWorker, input_text: String, option: SynthesisOption) -> Self {
    Self::Uninitialized(worker, input_text, option)
  }

  pub fn initialize(&mut self) -> SyrinxResult<()> {
    match self {
      Self::Initialized(_, _) => {}
      Self::Uninitialized(worker, input_text, option) => {
        let (generator, encoder) = worker.prepare(input_text, option)?;
        *self = Self::Initialized(generator, encoder);
      }
    }

    Ok(())
  }

  pub fn synthesize(&mut self) -> SyrinxResult<Vec<u8>> {
    match self {
      Self::Uninitialized(_, _, _) => {
        self.initialize()?;
        self.synthesize()
      }
      Self::Initialized(generator, encoder) => encoder.generate(generator),
    }
  }
}
