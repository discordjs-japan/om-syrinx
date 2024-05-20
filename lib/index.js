// @ts-check

const {
  EncoderType,
  Channels,
  Application,
  OmSyrinx: OmSyrinxInner,
} = require(`./native`);
const { Readable } = require("node:stream");

exports.EncoderType = EncoderType;
exports.Channels = Channels;
exports.Application = Application;

class OmSyrinxStream extends Readable {
  /**
   * @param {import("./native").OmSyrinx} syrinx
   * @param {boolean} objectMode
   * @param {string} inputText
   * @param {import("./native").SynthesisOption} option
   */
  constructor(syrinx, objectMode, inputText, option) {
    super({ objectMode });
    /**
     * @type {import("./native").OmSyrinx}
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
    setImmediate(() => this._read());
  }

  _read() {
    const cache = this._cache.shift();
    if (cache) this.push(cache);
    else if (this._ended) this.push(null);
    else this._waiting++;
  }
}

class OmSyrinx {
  /**
   *
   * @param {import("./native").OmSyrinxConfig} config
   * @returns {OmSyrinx}
   */
  static fromConfig(config) {
    const inner = OmSyrinxInner.fromConfig(config);
    return new OmSyrinx(inner, config.encoder.type === EncoderType.Opus);
  }

  /**
   * @param {import("./native").OmSyrinx} inner
   * @param {boolean} objectMode
   * @private
   */
  constructor(inner, objectMode) {
    /**
     * @type {import("./native").OmSyrinx}
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
    return new OmSyrinxStream(this._inner, this._objectMode, inputText, option);
  }
}

exports.OmSyrinx = OmSyrinx;
