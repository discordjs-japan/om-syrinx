# om-syrinx

om-syrinx（読み方：おーむ・しーりんくす）は，Discordの読み上げボット「[om](https://github.com/discordjs-japan/om)」のために作られた，日本語音声合成ライブラリです．

om-syrinxでは，入力全体を一度に処理するのではなく，順番に少量ずつ合成と出力を行います（準リアルタイム音声合成）．これにより，テキストが入力されてから音声が再生できるまでの時間が短くなるため，テキストチャンネルに投稿されたメッセージをいち早く読み上げはじめることができます．

実際のテキスト処理と音声合成はそれぞれ「[jpreprocess](https://crates.io/crates/jpreprocess)」と「[jbonsai](https://crates.io/crates/jbonsai)」が担っています．om-syrinxはこれらとNode.jsとのバインディングに加え，スレッド管理，バッファリング，opusへのエンコード機能を提供します．

## クイックスタート

### 準備

以下の3つの手順を実行してください．
- ライブラリ本体をインストールする：

  `npm install github:discordjs-japan/om-syrinx#v0.3.0`を実行してください．
- jpreprocess用の辞書をダウンロードする：

  [jpreprocessの最新のリリース](https://github.com/jpreprocess/jpreprocess/releases/v0.8.1)から[辞書 (`naist-jdic-jpreprocess.tar.gz`)](https://github.com/jpreprocess/jpreprocess/releases/download/v0.8.1/naist-jdic-jpreprocess.tar.gz) をダウンロードし，カレントディレクトリに解凍してください．
- jbonsai用のモデルをダウンロードする：

  htsvoice-tohoku-f01の`master`ブランチ (<https://github.com/icn-lab/htsvoice-tohoku-f01/archive/refs/heads/master.tar.gz>) をダウンロードし，カレントディレクトリに解凍してください．

<details>
  <summary>jbonsai用のモデルについて</summary>

  jbonsaiは，[HTS Engine](https://hts-engine.sourceforge.net)でも用いられる`.htsvoice`モデルを使用して音声を合成します．

  ここでは例として，[htsvoice-tohoku-f01](https://github.com/icn-lab/htsvoice-tohoku-f01) を使用しました．htsvoice-tohoku-f01は，4つの`.htsvoice`モデルを含むリポジトリです．他の`.htsvoice`モデルを使用することもできます．
</details>

### 使い方

ここでは，`inputText`から`stream`を生成する例を示します．

```ts
import { Syrinx, EncoderType, type SynthesisOption } from "@discordjs-japan/om-syrinx";
import { Readable } from "node:stream";

// インスタンスを生成
const syrinx = Syrinx.fromConfig({
  dictionary: "naist-jdic",
  models: ["htsvoice-tohoku-f01-master/tohoku-f01-neutral.htsvoice"],
  encoder: { type: EncoderType.Opus },
});

// 音声を合成
const inputText = "鳴管は、鳥類のもつ発声器官。";
const option: SynthesisOption = {};
const stream: Readable = syrinx.synthesize(inputText, option);

// @discordjs/voice で利用
import { createAudioResource, StreamType } from "@discordjs/voice";

const resource = createAudioResource(stream, { inputType: StreamType.Opus });
```

`Syrinx.fromConfig()`でインスタンスを生成する際に必須の設定は以下の通りです：
- `dictionary`：[jpreprocess用の辞書](#jpreprocess用の辞書)のフォルダのパス
- `models`：[jbonsai用のモデル](#jbonsai用のモデル)の`.htsvoice`ファイルのパスの配列
- `encoder`：エンコード設定
  - `EncoderType.Opus`の場合，Opusでエンコードされます．`@discordjs/voice`の`StreamType.Opus`に対応します．
  - `EncoderType.Raw`の場合，16ビットPCMに変換されます．`@discordjs/voice`の`StreamType.Raw`に対応します．
  
  その他の設定については，`EncoderConfig`を参照してください．

`syrinx.synthesize()`で音声を合成する際に渡す引数は以下の通りです：
- `inputText`：合成するテキスト
- `option`：合成される音声を調整するオプション．詳しくは，`SynthesisOption`を参照してください．

返り値の`stream`は`Readable`で，`encoder`設定の通りにエンコードされた音声データが流れます．
- `encoder.type`が`EncoderType.Opus`の場合，`stream`は object mode の`Readable`で，1つのオブジェクトが1つのOpusフレームに対応します．
- `encoder.type`が`EncoderType.Raw`の場合，`stream`は通常 (non-object mode) の`Readable`で，16ビットPCMのデータが流れます．

合成はメインスレッドとは別のスレッドで行われます．メインスレッドは出力を非同期に受け取ります．
