// @ts-check

/**
 * @fileoverview run following commands before test:
 * ```bash
 * curl -L https://github.com/jpreprocess/jpreprocess/releases/download/v0.8.1/naist-jdic-jpreprocess.tar.gz | tar zx
 * curl -L https://github.com/icn-lab/htsvoice-tohoku-f01/archive/refs/heads/master.zip | tar zx
 * ```
 */

const { it } = require("node:test");
const { AltJTalk, EncoderType } = require("../lib");
const path = require("path");
const assert = require("assert");

/**
 *
 * @param {AltJTalk} altJTalk
 * @param {string} inputText
 * @param {import("../lib").SynthesisOption} option
 * @returns {Promise<Buffer[]>}
 */
async function synthesize(altJTalk, inputText, option) {
  /** @type {Buffer[]} */
  const frames = [];

  return new Promise((resolve, reject) => {
    altJTalk
      .synthesize(inputText, option, (err, frame) => {
        if (err) reject(err);
        else frames.push(frame);
      })
      .then(() => resolve(frames), reject);
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
    encoder: { type: EncoderType.Pcm },
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
