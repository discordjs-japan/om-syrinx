const { filename } = require("./filename");
const {
  EncoderType,
  Channels,
  Application,
  OmSyrinx,
} = require(`../${filename}`);

module.exports.EncoderType = EncoderType;
module.exports.Channels = Channels;
module.exports.Application = Application;
module.exports.OmSyrinx = OmSyrinx;
