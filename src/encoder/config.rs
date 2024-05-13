/// Which encoder to use.
#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum EncoderType {
  /// Encodes audio data with Opus codec.
  /// One pushed chunk corresponds to one encoded frame.
  Opus,
  /// Returns raw s16le PCM data.
  /// One pushed chunk size is specified by [`EncoderConfig#chunk_size`].
  Raw,
}

/// Number of channels for generated audio.
#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum Channels {
  /// Generates mono audio.
  Mono = 1,
  /// Generates stereo audio with same data on both channels.
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

/// Opus encoder mode(application).
#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum Application {
  /// [`audiopus::Application::Voip`]
  Voip,
  /// [`audiopus::Application::Audio`]
  Audio,
  /// [`audiopus::Application::LowDelay`]
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

/// Encoder configuration.
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
