#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum EncoderType {
  Opus,
  Pcm,
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum Channels {
  Mono = 1,
  Stereo = 2,
}

impl From<Channels> for audiopus::Channels {
  fn from(channels: Channels) -> Self {
    match channels {
      Channels::Mono => Self::Mono,
      Channels::Stereo => Self::Stereo,
    }
  }
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum Application {
  Voip,
  Audio,
  LowDelay,
}

impl From<Application> for audiopus::Application {
  fn from(application: Application) -> Self {
    match application {
      Application::Voip => Self::Voip,
      Application::Audio => Self::Audio,
      Application::LowDelay => Self::LowDelay,
    }
  }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct EncoderConfig {
  /// Encoder type
  pub r#type: EncoderType,
  /// Number of channels.
  /// Used in type: Opus, Pcm
  pub channels: Option<Channels>,
  /// Opus encoder mode(application).
  /// Used in type: Opus
  pub mode: Option<Application>,
  /// Synthesized frame count per one chunk.
  /// Used in type: Pcm
  pub chunk_size: Option<u32>,
}
