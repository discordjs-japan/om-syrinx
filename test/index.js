// @ts-check

const { it, describe, before } = require("node:test");
const { AltJTalk, EncoderType } = require("../lib");
const path = require("node:path");
const assert = require("node:assert");
const zlib = require("node:zlib");
const fs = require("node:fs");
const { Readable } = require("node:stream");
const { pipeline } = require("node:stream/promises");
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
 *
 * @param {AltJTalk} altJTalk
 * @param {string} inputText
 * @param {import("../lib").SynthesisOption} option
 * @returns {Promise<Buffer[]>}
 */
async function synthesize(altJTalk, inputText, option) {
  const stream = altJTalk.synthesize(inputText, option);
  return new Promise((resolve, reject) => {
    const bufs = [];
    stream.on("data", (d) => bufs.push(d));
    stream.on("error", (err) => reject(err));
    stream.on("end", () => resolve(bufs));
  });
}

/**
 * @param {Buffer} buffer
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
    const pcm = AltJTalk.fromConfig({
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
    const opus = AltJTalk.fromConfig({
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

    await checksum(bonsai[25], "26a34471a9a047f9184494b7063699a3b0bc4771");
    await checksum(bonsai[50], "961959780d81dc24aea2b2c87f6617d12aabf99c");
    await checksum(bonsai[75], "b805476147d1da6b9c1eb9ede8a7a607d7accfa8");

    const isBonsai = await synthesize(opus, "これは，盆栽ですか？", {});
    assert.strictEqual(isBonsai.length, 136);

    await checksum(isBonsai[40], "9b93326876d0516d808c8bcafbd03221c2f3fbb7");
    await checksum(isBonsai[80], "10d56cbc58ba71627418a4305e41c454b623711b");
    await checksum(isBonsai[120], "b805476147d1da6b9c1eb9ede8a7a607d7accfa8");
  });
});
