// @ts-check

const {
  EncoderType,
  Channels,
  Application,
  AltJTalk: AltJTalkInner,
} = require(`./native`);
const { Readable } = require("node:stream");

exports.EncoderType = EncoderType;
exports.Channels = Channels;
exports.Application = Application;

class AltJTalkStream extends Readable {
  /**
   * @param {import("./native").AltJTalk} altJTalk
   * @param {boolean} objectMode
   * @param {string} inputText
   * @param {import("./native").SynthesisOption} option
   */
  constructor(altJTalk, objectMode, inputText, option) {
    super({ objectMode });
    /**
     * @type {import("./native").AltJTalk}
     * @private
     */
    this._altJTalk = altJTalk;
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
    const [synthesizer, error] = await this._altJTalk
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

class AltJTalk {
  /**
   *
   * @param {import("./native").AltJTalkConfig} config
   * @returns {AltJTalk}
   */
  static fromConfig(config) {
    const inner = AltJTalkInner.fromConfig(config);
    return new AltJTalk(inner, config.encoder.type === EncoderType.Opus);
  }

  /**
   * @param {import("./native").AltJTalk} inner
   * @param {boolean} objectMode
   * @private
   */
  constructor(inner, objectMode) {
    /**
     * @type {import("./native").AltJTalk}
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
    return new AltJTalkStream(this._inner, this._objectMode, inputText, option);
  }
}

exports.AltJTalk = AltJTalk;
