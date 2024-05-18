//@ts-check

const fs = require("node:fs");
const { Writable } = require("node:stream");
const { filename, url } = require("./filename");

// If this program is running on node-altjtalk-binding repo,
// do not download prebuilt binary
try {
  fs.accessSync(".git");
  process.exit(0);
} catch {}

fetch(url).then((response) => {
  if (!response.ok) {
    throw new Error(
      `Fetch failed.\nRequest to ${url} returned status code ${response.status}.`,
    );
  }

  response.body?.pipeTo(Writable.toWeb(fs.createWriteStream(filename)));
});
