# om-syrinx

om-syrinx（読み方：おーむ・しーりんくす）は，読み上げボット「[om](https://github.com/discordjs-japan/om)」のために作られた，音声合成ライブラリです．

実際のテキスト処理と音声合成はそれぞれ「[jpreprocess](https://crates.io/crates/jpreprocess)」と「[jbonsai](https://crates.io/crates/jbonsai)」が担っています．このリポジトリはこれらとNode.jsとのバインディングに加え，スレッド管理，バッファリング，opusへのエンコード機能を提供します．

## Cross compilation settings

To avoid linker error, write the following to your `~/.cargo/config.toml`
(if your native environment is `x86_64-unknown-linux-gnu`).

```toml
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
rustflags = ["-C", "target-feature=-crt-static"]

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```
