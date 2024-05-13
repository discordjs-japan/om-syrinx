use std::iter;

use jbonsai::{engine::Condition, speech::SpeechGenerator};

use super::{Channels, Encoder, EncoderConfig};

pub(super) struct PcmEncoder {
  channels: Channels,
  chunk_size: usize,
  speech: Vec<f64>,
}

impl PcmEncoder {
  pub fn speech_len(&self) -> usize {
    self.speech.len()
  }

  pub fn generate_i16<'a>(
    &'a mut self,
    generator: &mut SpeechGenerator,
  ) -> impl Iterator<Item = i16> + 'a {
    let fperiod = generator.fperiod();
    let mut generated = 0;
    for i in 0..self.chunk_size {
      generated += generator.generate_step(&mut self.speech[i * fperiod..(i + 1) * fperiod]);
    }
    self.speech[generated..].fill(0.0);

    self.speech.iter().flat_map(|f| {
      iter::repeat(f.clamp(i16::MIN as f64, i16::MAX as f64) as i16).take(self.channels as usize)
    })
  }
}

impl Encoder for PcmEncoder {
  fn new(condition: &Condition, config: &EncoderConfig) -> napi::Result<Self> {
    let channels = config.channels.unwrap_or(Channels::Stereo);
    let chunk_size = config.chunk_size.unwrap_or(4) as usize;
    let speech = vec![0.0; condition.get_fperiod() * chunk_size];
    Ok(Self {
      channels,
      chunk_size,
      speech,
    })
  }

  fn generate(&mut self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    let result = self
      .generate_i16(generator)
      .flat_map(|i| i.to_le_bytes())
      .collect();
    Ok(result)
  }
}
