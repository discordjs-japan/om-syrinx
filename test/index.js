// @ts-check

const { it, describe, before } = require("node:test");
const path = require("node:path");
const assert = require("node:assert");
const zlib = require("node:zlib");
const fs = require("node:fs");
const { Readable } = require("node:stream");
const { pipeline } = require("node:stream/promises");
const { setTimeout } = require("node:timers/promises");
const crypto = require("node:crypto");
const { Syrinx, EncoderType } = require("../lib");
const tar = require("tar-fs");

/**
 *
 * @param {string} url
 * @param {string} path
 * @returns {Promise<void>}
 */
async function fetchAndExtract(url, path) {
  const res = await fetch(url);
  if (!res.ok || !res.body)
    throw new Error(`Failed to fetch dictionary:\n${await res.text()}`);

  await pipeline([
    Readable.fromWeb(
      /** @type {import("node:stream/web").ReadableStream} */ (res.body),
    ),
    zlib.createGunzip(),
    tar.extract(path),
  ]);
}

/**
 * @param {Syrinx} syrinx
 * @param {string} inputText
 * @param {import("../lib").SynthesisOption} option
 * @returns {Promise<Buffer[]>}
 */
async function synthesize(syrinx, inputText, option) {
  const stream = syrinx.synthesize(inputText, option);
  /** @type {Buffer[]} */
  const result = [];
  for await (const item of stream) {
    result.push(item);
    await setTimeout(20);
  }
  return result;
}

/**
 * @param {BufferSource} buffer
 * @param {string} checksum
 */
async function checksum(buffer, checksum) {
  const hash = await crypto.subtle.digest("SHA-1", buffer);
  assert.strictEqual(Buffer.from(hash).toString("hex"), checksum);
}

describe("synthesis", () => {
  before(async () => {
    if (!fs.existsSync(path.join(__dirname, "naist-jdic"))) {
      await fetchAndExtract(
        "https://github.com/jpreprocess/jpreprocess/releases/download/v0.8.1/naist-jdic-jpreprocess.tar.gz",
        __dirname,
      );
    }
    if (!fs.existsSync(path.join(__dirname, "htsvoice-tohoku-f01-master"))) {
      await fetchAndExtract(
        "https://github.com/icn-lab/htsvoice-tohoku-f01/archive/refs/heads/master.tar.gz",
        __dirname,
      );
    }
  });

  it("should synthesize identical PCM data", async () => {
    const pcm = Syrinx.fromConfig({
      dictionary: path.join(__dirname, "naist-jdic"),
      models: [
        path.join(
          __dirname,
          "htsvoice-tohoku-f01-master",
          "tohoku-f01-neutral.htsvoice",
        ),
      ],
      encoder: { type: EncoderType.Raw },
    });

    const bonsai = await synthesize(pcm, "盆栽", {});
    await checksum(
      Buffer.concat(bonsai),
      "36050aef60896f56bbfe59868a3f57b6bbc5b147",
    );

    const isBonsai = await synthesize(pcm, "これは，盆栽ですか？", {});
    await checksum(
      Buffer.concat(isBonsai),
      "c0e3f8773a47534862f7e553fb00d48022496b19",
    );
  });

  it("should synthesize identical Opus data", async () => {
    const { OpusDecoder } = await import("opus-decoder");
    const decoder = new OpusDecoder();
    await decoder.ready;

    const opus = Syrinx.fromConfig({
      dictionary: path.join(__dirname, "naist-jdic"),
      models: [
        path.join(
          __dirname,
          "htsvoice-tohoku-f01-master",
          "tohoku-f01-neutral.htsvoice",
        ),
      ],
      encoder: { type: EncoderType.Opus },
    });

    const bonsai = await synthesize(opus, "盆栽", {});
    assert.strictEqual(bonsai.length, 78);

    const bonsaiDecoded = decoder.decodeFrames(bonsai);
    assert.strictEqual(bonsaiDecoded.sampleRate, 48000);
    assert.strictEqual(bonsaiDecoded.samplesDecoded, 74880);
    assert.strictEqual(bonsaiDecoded.channelData.length, 2);

    // Approximate by converting to Int16Array
    await checksum(
      new Int16Array(bonsaiDecoded.channelData[0]),
      "c4a707f5f150cf69ed8f9dd0068075f8ff96b2e7",
    );
    await checksum(
      new Int16Array(bonsaiDecoded.channelData[1]),
      "c4a707f5f150cf69ed8f9dd0068075f8ff96b2e7",
    );

    await decoder.reset();

    const isBonsai = await synthesize(opus, "これは，盆栽ですか？", {});
    assert.strictEqual(isBonsai.length, 136);

    const isBonsaiDecoded = decoder.decodeFrames(isBonsai);
    assert.strictEqual(isBonsaiDecoded.sampleRate, 48000);
    assert.strictEqual(isBonsaiDecoded.samplesDecoded, 130560);
    assert.strictEqual(isBonsaiDecoded.channelData.length, 2);

    // Approximate by converting to Int16Array
    await checksum(
      new Int16Array(isBonsaiDecoded.channelData[0]),
      "97bca4f1bbcd59e65a14a1c739db895736dd9528",
    );
    await checksum(
      new Int16Array(isBonsaiDecoded.channelData[1]),
      "97bca4f1bbcd59e65a14a1c739db895736dd9528",
    );

    await decoder.reset();
    decoder.free();
  });
});
