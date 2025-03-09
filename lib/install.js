const fs = require("node:fs");
const { Readable } = require("node:stream");
const { filename, url } = require("./filename");

// If this program is running on om-syrinx repo,
// do not download prebuilt binary
try {
  fs.accessSync(".git");
  process.exit(0);
} catch {}

fetch(url).then((response) => {
  if (!response.ok || !response.body) {
    throw new Error(
      `Fetch failed.\nRequest to ${url} returned status code ${response.status}.`,
    );
  }

  Readable.fromWeb(response.body).pipe(fs.createWriteStream(filename));
});
