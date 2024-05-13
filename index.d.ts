export { EncoderType, Channels, Application } from "./native";
export type {
  EncoderConfig,
  SynthesisOption,
  InterporationWeight,
  AltJTalkConfig,
} from "./native";

import type { Readable } from "node:stream";
import type { AltJTalkConfig, SynthesisOption } from "./native";

/** Text-to-speech engine with multi-threading support backed by libuv. */
export class AltJTalk {
  /** Create a new instance of `AltJTalk` with the given configuration. */
  static fromConfig(config: AltJTalkConfig): AltJTalk;
  /**
   * Start synthesis with the given input text and option on the libuv worker thread.
   *
   * @param inputText Input text to synthesize.
   * @param option Synthesis option.
   * @returns Readable stream of either Raw or PCM audio data.
   */
  synthesize(inputText: string, option: SynthesisOption): Readable;
}
