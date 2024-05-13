/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

/** Which encoder to use. */
export const enum EncoderType {
  /**
   * Encodes audio data with Opus codec.
   * One pushed chunk corresponds to one encoded frame.
   */
  Opus = 0,
  /**
   * Returns raw s16le PCM data.
   * One pushed chunk size is specified by [`EncoderConfig#chunk_size`].
   */
  Raw = 1
}
/** Number of channels for generated audio. */
export const enum Channels {
  /** Generates mono audio. */
  Mono = 1,
  /** Generates stereo audio with same data on both channels. */
  Stereo = 2
}
/** Opus encoder mode(application). */
export const enum Application {
  /** [`audiopus::Application::Voip`] */
  Voip = 0,
  /** [`audiopus::Application::Audio`] */
  Audio = 1,
  /** [`audiopus::Application::LowDelay`] */
  LowDelay = 2
}
/** Encoder configuration. */
export interface EncoderConfig {
  /** Encoder type */
  type: EncoderType
  /**
   * Number of channels.
   * Used in type: Opus, Pcm
   */
  channels?: Channels
  /**
   * Opus encoder mode(application).
   * Used in type: Opus
   */
  mode?: Application
  /**
   * Synthesized frame count per one chunk.
   * Used in type: Pcm
   */
  chunkSize?: number
}
/** Voice synthesis option. */
export interface SynthesisOption {
  /**
   * Frequency warping parameter alpha
   * 0.0<=all_pass_constant<=1.0.
   */
  allPassConstant?: number
  /**
   * Postfiltering coefficient parameter beta
   * Default is 0.0. 0.0<=postfiltering_coefficient<=1.0.
   */
  postfilteringCoefficient?: number
  /**
   * Speech speed
   * Default is 1.0. 0<=speech_speed_rate. Warning: Do not set a very small value as it consumes CPU time.
   */
  speechSpeedRate?: number
  /**
   * Additional half tone
   * Default is 0.0.
   */
  additionalHalfTone?: number
  /**
   * MSD threshold for Stream #1
   * Default is 0.5. 0.0<=voiced_unvoiced_threshold<=1.0.
   */
  voicedUnvoicedThreshold?: number
  /**
   * GV weight for Stream #0
   * Default is 1.0. 0.0<=weight_of_gv_for_spectrum.
   */
  weightOfGvForSpectrum?: number
  /**
   * GV weight for Stream #1
   * Default is 1.0. 0.0<=weight_of_gv_for_log_f0.
   */
  weightOfGvForLogF0?: number
  /**
   * Volume in dB
   * Default is 0.0.
   */
  volumeInDb?: number
  /** Interporation weights */
  interporationWeight?: InterporationWeight
  /**
   * Shorthand for [`SynthesisOption::interporation_weight`].
   * Note that this option is ignored if [`SynthesisOption::interporation_weight`] exists.
   */
  voice?: number
}
/**
 * How loaded models are mixed.
 *
 * All weight array must:
 * - be same length as loadad models.
 * - have values between 0.0 and 1.0.
 * - sum up to 1.0.
 */
export interface InterporationWeight {
  /** Duration */
  duration?: Array<number>
  /** Stream #0 */
  spectrum?: Array<number>
  /** Stream #1 */
  logF0?: Array<number>
  /** Stream #2 */
  lpf?: Array<number>
}
/** Configuration for `AltJTalk`. */
export interface AltJTalkConfig {
  /** Dictionary file path. */
  dictionary: string
  /** User dictionary file path. */
  userDictionary?: string
  /** Model file paths. */
  models: Array<string>
  /** Encoder configuration. */
  encoder: EncoderConfig
}
/** Text-to-speech engine with multi-threading support backed by libuv. */
export class AltJTalk {
  /** Create a new instance of `AltJTalk` with the given configuration. */
  static fromConfig(config: AltJTalkConfig): AltJTalk
  /**
   * Start synthesis with the given input text and option on the libuv worker thread.
   *
   * @param inputText Input text to synthesize.
   * @param option Synthesis option.
   * @param push Callback function to push synthesized frames. The content of the buffer depends on {@link EncoderConfig}.
   */
  synthesize(inputText: string, option: SynthesisOption, push: (...args: [err: null, frame: Buffer] | [err: Error, frame: null]) => void): Promise<unknown>
}
