use jbonsai::{engine::Condition, speech::SpeechGenerator};

use super::{pcm::PcmEncoder, Application, Channels, Encoder};

#[derive(Debug)]
pub struct OpusEncoder {
  pcm_encoder: PcmEncoder,
  opus_encoder: audiopus::coder::Encoder,
}

const OPUS_FRAME_SIZE: usize = 20;

impl OpusEncoder {
  pub fn new(
    condition: &Condition,
    channels: Channels,
    mode: Application,
  ) -> Result<Self, audiopus::Error> {
    let sample_rate = audiopus::SampleRate::try_from(condition.get_sampling_frequency() as i32)?;
    let chunk_size = sample_rate as usize / condition.get_fperiod() * OPUS_FRAME_SIZE / 1000;

    Ok(Self {
      pcm_encoder: PcmEncoder::new(channels, chunk_size),
      opus_encoder: audiopus::coder::Encoder::new(sample_rate, channels.into(), mode.into())?,
    })
  }
}

impl Encoder for OpusEncoder {
  fn generate(&self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    let pcm: Vec<_> = self.pcm_encoder.generate_i16(generator).collect();
    let mut output = vec![0; pcm.len()];
    let size = self
      .opus_encoder
      .encode(&pcm, &mut output)
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;
    output.resize(size, 0);
    Ok(output)
  }
}
