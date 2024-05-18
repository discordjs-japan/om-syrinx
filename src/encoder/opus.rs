use jbonsai::{engine::Condition, speech::SpeechGenerator};

use super::{pcm::PcmEncoder, Application, Channels, Encoder, EncoderConfig, EncoderType};

pub struct OpusEncoder {
  channels: Channels,
  pcm_encoder: PcmEncoder,
  opus_encoder: audiopus::coder::Encoder,
  output: Vec<u8>,
}

const OPUS_FRAME_SIZE: usize = 20; // ms

impl Encoder for OpusEncoder {
  fn new(condition: &Condition, config: &EncoderConfig) -> napi::Result<Self>
  where
    Self: Sized,
  {
    let sample_rate = audiopus::SampleRate::try_from(condition.get_sampling_frequency() as i32)
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;
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
    let opus_encoder = audiopus::coder::Encoder::new(sample_rate, channels.into(), mode.into())
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;

    Ok(Self {
      output: vec![0; pcm_encoder.speech_len() * channels as usize], // TODO: better capacity estimation
      channels,
      pcm_encoder,
      opus_encoder,
    })
  }

  fn generate(&mut self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    let mut pcm: Vec<_> = self.pcm_encoder.generate_i16(generator).collect();
    if pcm.is_empty() {
      return Ok(vec![]);
    }
    pcm.resize(self.pcm_encoder.speech_len() * self.channels as usize, 0);

    let size = self
      .opus_encoder
      .encode(&pcm, &mut self.output)
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;

    Ok(self.output[..size].to_vec())
  }
}
