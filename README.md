# om-syrinx

om-syrinx（読み方：おーむ・しーりんくす）は，Discordの読み上げボット「[om](https://github.com/discordjs-japan/om)」のために作られた，音声合成ライブラリです．

実際のテキスト処理と音声合成はそれぞれ「[jpreprocess](https://crates.io/crates/jpreprocess)」と「[jbonsai](https://crates.io/crates/jbonsai)」が担っています．このリポジトリはこれらとNode.jsとのバインディングに加え，スレッド管理，バッファリング，opusへのエンコード機能を提供します．

## インストール

### ライブラリ本体

1.  `package.json`の`dependencies`に以下を追加します：
    ```
    "@discordjs-japan/om-syrinx": "github:discordjs-japan/om-syrinx#v0.3.0"
    ```
    ただし，バージョンは適宜変更してください．
2.  `npm install`を実行します．

### jpreprocess用の辞書

1.  [`jperprocess`のリリース](https://github.com/jpreprocess/jpreprocess/releases)から最新の辞書 (`naist-jdic-jpreprocess.tar.gz`) をダウンロードします．
1.  `tar xz`等で展開して，`naist-jdic`フォルダができることを確認します．

### jbonsai用のモデル

ここでは，[`htsvoice-tohoku-f01`](https://github.com/icn-lab/htsvoice-tohoku-f01)をダウンロードします．`htsvoice-tohoku-f01`は，4種類の声色のモデルを含むリポジトリです．他のモデルを利用することもできます．

1.  `htsvoice-tohoku-f01`の`master`ブランチ (<https://github.com/icn-lab/htsvoice-tohoku-f01/archive/refs/heads/master.tar.gz>) をダウンロードします．
1.  `tar xz`等で展開して，`htsvoice-tohoku-f01-master`フォルダができることを確認します．

## 使い方

ここでは，`inputText`から`stream`を生成する例を示します．

```ts
import { Syrinx, EncoderType, type SynthesisOption } from "@discordjs-japan/om-syrinx";

// インスタンスを生成
const syrinx = Syrinx.fromConfig({
  dictionary: "naist-jdic",
  models: ["htsvoice-tohoku-f01-master/tohoku-f01-neutral.htsvoice"],
  encoder: { type: EncoderType.Opus },
});

// 音声を合成
const inputText = "鳴管は、鳥類のもつ発声器官。";
const option: SynthesisOption = {};
const stream = syrinx.synthesize(inputText, option);

// @discordjs/voice で利用
import { createAudioResource, StreamType } from "@discordjs/voice";

const resource = createAudioResource(stream, { inputType: StreamType.Opus });
```

インスタンスを生成する際に必須の設定は以下の通りです：
- `dictionary`：[`jpreprocess`用の辞書](#jpreprocess用の辞書)のフォルダのパス
- `models`：[`jbonsai`用のモデル](#jbonsai用のモデル)の`.htsvoice`ファイルのパスの配列
- `encoder`：エンコード設定．エンコードは，`@discordjs/voice`の`createAudioResource`に渡すことを念頭に行われています．
  - `EncoderType.Opus`の場合，オブジェクトモードで，1つのオブジェクトが1つのOpusフレームに対応します．`@discordjs/voice`の`StreamType.Opus`に対応します．
  - `EncoderType.Raw`の場合，16ビットPCMデータが流れます．`@discordjs/voice`の`StreamType.Raw`に対応します．
  
  その他の設定については，`EncoderConfig`を参照してください．

音声を合成する際に渡す引数は以下の通りです：
- `inputText`：合成するテキスト
- `option`：合成される音声を調整するオプション．詳しくは，`SynthesisOption`を参照してください．

返り値の`stream`は`Readable`で，`encoder`設定の通りにエンコードされた音声データが流れます．合成はメインスレッドとは別のスレッドで行われます．また，音声は合成された分から合成終了を待たずに取得できます．

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
