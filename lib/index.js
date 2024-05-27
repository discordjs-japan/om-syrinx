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

const { setTimeout } = require("node:timers/promises");

class Syrinx {
  /**
   * @param {import("./native").SyrinxConfig} config
   * @returns {Syrinx}
   */
  static fromConfig(config) {
    const inner = SyrinxInner.fromConfig(config);
    return new Syrinx(inner, config.encoder.type === EncoderType.Opus);
  }

  /**
   * @param {import("./native").SyrinxConfig} config
   * @returns {Promise<Syrinx>}
   */
  static async fromConfigAsync(config) {
    const inner = await SyrinxInner.fromConfigAsync(config);
    return new Syrinx(inner, config.encoder.type === EncoderType.Opus);
  }

  /**
   * @param {import("./native").Syrinx} inner
   * @param {boolean} objectMode
   * @private
   */
  constructor(inner, objectMode) {
    /**
     * @type {import("./native").Syrinx}
     * @private
     */
    this._inner = inner;
    /**
     * @type {boolean}
     * @private
     */
    this._objectMode = objectMode;
  }

  /**
   * @param {string} inputText
   * @param {import("./native").SynthesisOption} option
   * @returns {Readable}
   */
  synthesize(inputText, option) {
    const { _inner } = this;

    return new Readable({
      objectMode: this._objectMode,

      /**
       * @param {(error?: Error | null) => void} callback
       */
      async construct(callback) {
        /** @type {[import("./native").PreparedSynthesizer, null] | [null, Error]} */
        const [synthesizer, error] = await _inner
          .prepare(inputText, option)
          .then(
            (s) => [s, null],
            (e) => [null, e],
          );
        callback(error);

        await synthesizer?.synthesize((err, result) => {
          if (err) this.emit("error", err);
          else this.push(result);
        });

        // The last push sometimes occurs after the Promise resolution,
        // so we need to make sure that the last push occurs before the stream ends.
        await setTimeout();
        this.push(null);
      },

      read() {},
    });
  }
}

exports.Syrinx = Syrinx;
