use std::iter;

use jbonsai::speech::SpeechGenerator;

use super::{Channels, Encoder};

#[derive(Debug, Clone)]
pub struct PcmEncoder {
  channels: Channels,
  chunk_size: usize,
}

impl PcmEncoder {
  pub fn new(channels: Channels, chunk_size: usize) -> Self {
    Self {
      channels,
      chunk_size,
    }
  }

  pub(super) fn generate_i16<'a>(
    &'a self,
    generator: &mut SpeechGenerator,
  ) -> impl Iterator<Item = i16> + 'a {
    let fperiod = generator.fperiod();
    let mut speech = vec![0.0; fperiod * self.chunk_size];
    let mut generated = 0;
    for i in 0..self.chunk_size {
      generated += generator.generate_step(&mut speech[i * fperiod..(i + 1) * fperiod]);
    }

    speech.into_iter().take(generated).flat_map(|f| {
      iter::repeat(f.clamp(i16::MIN as f64, i16::MAX as f64) as i16).take(self.channels as usize)
    })
  }
}

impl Encoder for PcmEncoder {
  fn generate(&self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    Ok(
      self
        .generate_i16(generator)
        .flat_map(|i| i.to_le_bytes())
        .collect(),
    )
  }
}
