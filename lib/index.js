// @ts-check

const {
  JPREPROCESS_VERSION,
  JBONSAI_VERSION,
  EncoderType,
  Channels,
  Application,
  Syrinx: SyrinxInner,
} = require(`./native`);
const { Readable } = require("node:stream");

exports.JPREPROCESS_VERSION = JPREPROCESS_VERSION;
exports.JBONSAI_VERSION = JBONSAI_VERSION;
exports.EncoderType = EncoderType;
exports.Channels = Channels;
exports.Application = Application;

const { version: OM_SYRINX_VERSION } = require("../package.json");
exports.OM_SYRINX_VERSION = OM_SYRINX_VERSION;

class Syrinx {
  /**
   * @param {import("./native").SyrinxConfig} config
   * @returns {Syrinx}
   */
  static fromConfig(config) {
    const inner = SyrinxInner.fromConfig(config);
    return new Syrinx(inner);
  }

  /**
   * @param {import("./native").SyrinxConfig} config
   * @returns {Promise<Syrinx>}
   */
  static async fromConfigAsync(config) {
    const inner = await SyrinxInner.fromConfigAsync(config);
    return new Syrinx(inner);
  }

  /**
   * @param {import("./native").Syrinx} inner
   * @private
   */
  constructor(inner) {
    /**
     * @type {import("./native").Syrinx}
     * @private
     */
    this._inner = inner;
  }

  /**
   * @param {string} inputText
   * @param {import("./native").SynthesisOption} option
   * @returns {Readable}
   */
  synthesize(inputText, option) {
    const stream = this._inner.stream(inputText, option);
    return new Readable({
      objectMode: stream.objectMode,
      /**
       * @param {(error?: Error | null) => void} callback
       */
      construct(callback) {
        stream.construct().then(callback, callback);
      },
      /**
       * @param {number} _size
       */
      read(_size) {
        stream.read().then(
          (buf) => this.push(buf),
          (e) => this.emit("error", e),
        );
      },
    });
  }
}

exports.Syrinx = Syrinx;
