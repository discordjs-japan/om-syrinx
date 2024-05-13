const packageJSON = require("./package.json");
const { platform, arch } = process;

function linuxSuffix() {
  const report = process.report.getReport();
  const { header } = typeof report === "object" ? report : JSON.parse(report);
  return header.glibcVersionRuntime ? "gnu" : "musl";
}

/** @type {string[]} */
const triples = [platform, arch];
switch (platform) {
  case "win32":
    triples.push("msvc");
    break;
  case "linux":
    triples.push(linuxSuffix());
    break;
}

const filename = `${packageJSON.napi.name}.${triples.join("-")}.node`;
exports.filename = filename;

const repositoryUrl = new URL(packageJSON.repository.url);
const pathname = repositoryUrl.pathname.replace(/\.git$/, "");
const url = new URL(pathname, "https://github.com");
url.pathname = `${url.pathname}/releases/download/v${packageJSON.version}/${filename}`;
exports.url = url;
