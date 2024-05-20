export { EncoderType, Channels, Application } from "./native";
export type {
  EncoderConfig,
  SynthesisOption,
  InterporationWeight,
  OmSyrinxConfig,
} from "./native";

import type { Readable } from "node:stream";
import type { OmSyrinxConfig, SynthesisOption } from "./native";

/** Text-to-speech engine with multi-threading support backed by libuv. */
export class OmSyrinx {
  /** Create a new instance of `OmSyrinx` with the given configuration. */
  static fromConfig(config: OmSyrinxConfig): OmSyrinx;
  /**
   * Start synthesis with the given input text and option on the libuv worker thread.
   *
   * @param inputText Input text to synthesize.
   * @param option Synthesis option.
   * @returns Readable stream of either Opus (objectMode) or Raw audio data.
   */
  synthesize(inputText: string, option: SynthesisOption): Readable;
}
