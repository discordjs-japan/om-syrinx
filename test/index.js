const { it, describe, before } = require("node:test");
const path = require("node:path");
const assert = require("node:assert");
const zlib = require("node:zlib");
const fs = require("node:fs");
const { Readable } = require("node:stream");
const { buffer } = require("node:stream/consumers");
const { pipeline } = require("node:stream/promises");
const crypto = require("node:crypto");
const {
  OM_SYRINX_VERSION,
  JPREPROCESS_VERSION,
  JBONSAI_VERSION,
  Syrinx,
  EncoderType,
} = require("../lib");
const tar = require("tar-fs");
const TOML = require("smol-toml");

describe("version", () => {
  it("should match the version of om-syrinx", () => {
    assert.strictEqual(require("../package.json").version, OM_SYRINX_VERSION);
  });

  const lockFile = fs.readFileSync("Cargo.lock", "utf-8");
  const { package: packages } = TOML.parse(lockFile);

  assert(Array.isArray(packages));
  assert(
    packages.every(
      /** @returns {p is { name: string; version: string }} */
      (p) =>
        typeof p === "object" &&
        !!p &&
        "name" in p &&
        typeof p.name === "string" &&
        "version" in p &&
        typeof p.version === "string",
    ),
  );

  const jpreprocess = packages.find((p) => p.name === "jpreprocess");
  assert(jpreprocess);
  const jbonsai = packages.find((p) => p.name === "jbonsai");
  assert(jbonsai);

  it("should match the version of jpreprocess", () => {
    assert.strictEqual(JPREPROCESS_VERSION, jpreprocess.version);
  });

  it("should match the version of jbonsai", () => {
    assert.strictEqual(JBONSAI_VERSION, jbonsai.version);
  });
});

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
    Readable.fromWeb(res.body),
    zlib.createGunzip(),
    tar.extract(path),
  ]);
}

/**
 * @param {crypto.webcrypto.BufferSource} buffer
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
        `https://github.com/jpreprocess/jpreprocess/releases/download/v${JPREPROCESS_VERSION}/naist-jdic-jpreprocess.tar.gz`,
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

    const bonsai = await buffer(pcm.synthesize("盆栽", {}));
    assert.strictEqual(bonsai, "36050aef60896f56bbfe59868a3f57b6bbc5b147");

    const isBonsai = await buffer(pcm.synthesize("これは，盆栽ですか？", {}));
    assert.strictEqual(isBonsai, "c0e3f8773a47534862f7e553fb00d48022496b19");
  });

  it("should synthesize identical Opus data", async () => {
    const { OpusDecoder } = await import("opus-decoder");
    const decoder = new OpusDecoder();
    await decoder.ready;

    const opus = await Syrinx.fromConfigAsync({
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

    const bonsai = await opus.synthesize("盆栽", {}).toArray();
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

    const isBonsai = await opus
      .synthesize("これは，盆栽ですか？", {})
      .toArray();
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
