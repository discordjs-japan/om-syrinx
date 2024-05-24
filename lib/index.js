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

class SyrinxStream extends Readable {
  /**
   * @param {import("./native").Syrinx} syrinx
   * @param {boolean} objectMode
   * @param {string} inputText
   * @param {import("./native").SynthesisOption} option
   */
  constructor(syrinx, objectMode, inputText, option) {
    super({ objectMode });
    /**
     * @type {import("./native").Syrinx}
     * @private
     */
    this._syrinx = syrinx;
    /**
     * @type {string}
     * @private
     */
    this._inputText = inputText;
    /**
     * @type {import("./native").SynthesisOption}
     * @private
     */
    this._option = option;
    /**
     * @type {Buffer[]}
     * @private
     */
    this._cache = [];
    /**
     * @type {number}
     * @private
     */
    this._waiting = 0;
    /**
     * @type {boolean}
     * @private
     */
    this._ended = false;
  }

  /**
   * @param {(error?: Error | null) => void} callback
   */
  async _construct(callback) {
    /** @type {[import("./native").PreparedSynthesizer, null] | [null, Error]} */
    const [synthesizer, error] = await this._syrinx
      .prepare(this._inputText, this._option)
      .then(
        (s) => [s, null],
        (e) => [null, e],
      );
    callback(error);

    await synthesizer?.synthesize((err, result) => {
      if (err) return this.emit("error", err);
      if (this._waiting > 0) {
        this._waiting--;
        this.push(result);
      } else this._cache.push(result);
    });

    this._ended = true;
  }

  _read() {
    const cache = this._cache.shift();
    if (cache) this.push(cache);
    else if (this._ended) this.push(null);
    else this._waiting++;
  }
}

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
    return new SyrinxStream(this._inner, this._objectMode, inputText, option);
  }
}

exports.Syrinx = Syrinx;
