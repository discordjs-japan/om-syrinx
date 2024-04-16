//@ts-check

import fs from "node:fs/promises"
import path from "node:path";
import { fileURLToPath } from "node:url";
// If this program is running on node-altjtalk-binding repo,
// do not download prebuilt binary
const isRepo = await fs.access(".git").then(() => true, () => false);
if (isRepo) process.exit(0);

async function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return (await fs.readFile(lddPath, 'utf8')).includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const report = process.report.getReport();
    const { glibcVersionRuntime } = (typeof report === "object" ? report : JSON.parse(report)).header
    return !glibcVersionRuntime
  }
}

const baseUrl = "https://github.com/femshima/node-altjtalk-binding/releases/download";

const file = fileURLToPath(import.meta.url);
const dir = path.dirname(file);

const packageJSON = JSON.parse(
  await fs.readFile(path.join(dir, "package.json"), { encoding: "utf-8" })
);
const packageName = packageJSON.name ?? "node-altjtalk-binding";
const version = `v${packageJSON.version}` ?? "latest";

const { platform, arch } = process;

let suffix = "";
switch (platform) {
  case 'win32':
    suffix = "-msvc";
    break;
  case 'linux':
    suffix = (await isMusl()) ? "-musl" : "-gnu";
    break;
}

/**
 * @param {string} fileName
 */
async function download(fileName) {
  const url = `${baseUrl}/${version}/${fileName}`
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Fetch failed.\nRequest to ${url} returned status code ${response.status}.`);
  }

  const buf = await response.arrayBuffer();
  await fs.writeFile(fileName, Buffer.from(buf));
}

await Promise.all([
  download(`${packageName}.${platform}-${arch}${suffix}.node`),
  download("version.json"),
]);
