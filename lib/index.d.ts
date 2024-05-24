export {
  JPREPROCESS_VERSION,
  JBONSAI_VERSION,
  EncoderType,
  Channels,
  Application,
} from "./native";
export type {
  EncoderConfig,
  SynthesisOption,
  InterporationWeight,
  SyrinxConfig,
} from "./native";

export const OM_SYRINX_VERSION: string;

import type { Readable } from "node:stream";
import type { SyrinxConfig, SynthesisOption } from "./native";

/** Text-to-speech engine with multi-threading support backed by libuv. */
export class Syrinx {
  /** Create a new instance of `Syrinx` with the given configuration. */
  static fromConfig(config: SyrinxConfig): Syrinx;
  /**
   * Start synthesis with the given input text and option on the libuv worker thread.
   *
   * @param inputText Input text to synthesize.
   * @param option Synthesis option.
   * @returns Readable stream of either Opus (objectMode) or Raw audio data.
   */
  synthesize(inputText: string, option: SynthesisOption): Readable;
}
