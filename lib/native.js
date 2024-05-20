const { filename } = require("./filename");
const {
  EncoderType,
  Channels,
  Application,
  Syrinx,
} = require(`../${filename}`);

module.exports.EncoderType = EncoderType;
module.exports.Channels = Channels;
module.exports.Application = Application;
module.exports.Syrinx = Syrinx;
