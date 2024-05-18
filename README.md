# om-syrinx

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
