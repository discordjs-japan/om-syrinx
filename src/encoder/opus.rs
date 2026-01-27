use jbonsai::{engine::Condition, speech::SpeechGenerator};

use crate::error::SyrinxResult;

use super::{Application, Channels, Encoder, EncoderConfig, EncoderType, pcm::PcmEncoder};

pub struct OpusEncoder {
  channels: Channels,
  pcm_encoder: PcmEncoder,
  opus_encoder: opus2::Encoder,
  output: Vec<u8>,
}

const OPUS_FRAME_SIZE: usize = 20; // ms

impl Encoder for OpusEncoder {
  fn new(condition: &Condition, config: &EncoderConfig) -> SyrinxResult<Self>
  where
    Self: Sized,
  {
    let sample_rate = condition.get_sampling_frequency();
    let channels = config.channels.unwrap_or(Channels::Stereo);
    let mode = config.mode.unwrap_or(Application::Voip);

    let synthesis_per_second = condition.get_sampling_frequency() / condition.get_fperiod();
    let chunk_size = synthesis_per_second * OPUS_FRAME_SIZE / 1000;
    let pcm_config = EncoderConfig {
      r#type: EncoderType::Raw,
      channels: Some(channels),
      mode: None,
      chunk_size: Some(chunk_size as u32),
    };

    let pcm_encoder = PcmEncoder::new(condition, &pcm_config)?;
    let opus_encoder = opus2::Encoder::new(sample_rate as u32, channels.into(), mode.into())?;

    Ok(Self {
      // The maximum representable length is 255*4+255=1275 bytes.
      // https://datatracker.ietf.org/doc/html/rfc6716#section-3.2.1
      output: vec![0; 1275],
      channels,
      pcm_encoder,
      opus_encoder,
    })
  }

  fn generate(&mut self, generator: &mut SpeechGenerator) -> SyrinxResult<Vec<u8>> {
    let mut pcm: Vec<_> = self.pcm_encoder.generate_i16(generator).collect();
    if pcm.is_empty() {
      return Ok(vec![]);
    }
    pcm.resize(self.pcm_encoder.speech_len() * self.channels as usize, 0);

    let size = self.opus_encoder.encode(&pcm, &mut self.output)?;

    Ok(self.output[..size].to_vec())
  }
}
