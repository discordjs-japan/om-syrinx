# om-syrinx

om-syrinx（読み方：おーむ・しーりんくす）は，Discordの読み上げボット「[om](https://github.com/discordjs-japan/om)」のために作られた，音声合成ライブラリです．

om-syrinxでは，音声が前から順に合成され，最後まで合成されるのを待たずに出力されます．つまり，音声合成と再生を同時に行うことができます．ここでは，これを「ストリーミング合成」と呼びます．ストリーミング合成により，音声が再生できるまでの時間が短くなるため，テキストチャンネルに投稿されたメッセージをいち早く読み上げることができます．

実際のテキスト処理と音声合成はそれぞれ「[jpreprocess](https://crates.io/crates/jpreprocess)」と「[jbonsai](https://crates.io/crates/jbonsai)」が担っています．このリポジトリはこれらとNode.jsとのバインディングに加え，スレッド管理，バッファリング，opusへのエンコード機能を提供します．

## インストール

### ライブラリ本体

以下を実行してください：
```sh
npm install github:discordjs-japan/om-syrinx#v0.3.0
```
ただし，バージョンは適宜変更してください．

### jpreprocess用の辞書

1.  [`jperprocess`のリリース](https://github.com/jpreprocess/jpreprocess/releases)から最新の辞書 (`naist-jdic-jpreprocess.tar.gz`) をダウンロードしてください．
1.  `tar xz`等で展開して，生成される`naist-jdic`フォルダを適切な場所に配置してください．

### jbonsai用のモデル

jbonsaiは，[HTS Engine](https://hts-engine.sourceforge.net)でも用いられる`.htsvoice`モデルを使用して音声を合成します．
ここでは例として，[`htsvoice-tohoku-f01`](https://github.com/icn-lab/htsvoice-tohoku-f01)を使用します．`htsvoice-tohoku-f01`は，4つの`.htsvoice`モデルを含むリポジトリです．他の`.htsvoice`モデルを使用することもできます．

1.  `htsvoice-tohoku-f01`の`master`ブランチ (<https://github.com/icn-lab/htsvoice-tohoku-f01/archive/refs/heads/master.tar.gz>) をダウンロードしてください．
1.  `tar xz`等で展開して，生成される`htsvoice-tohoku-f01-master`フォルダを適切な場所に配置してください．

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
- `encoder`：エンコード設定．エンコードは，`@discordjs/voice`の`createAudioResource`に渡すことを想定して行われています．
  - `EncoderType.Opus`の場合，オブジェクトモードで，1つのオブジェクトが1つのOpusフレームに対応します．`@discordjs/voice`の`StreamType.Opus`に対応します．
  - `EncoderType.Raw`の場合，16ビットPCMデータが流れます．`@discordjs/voice`の`StreamType.Raw`に対応します．
  
  その他の設定については，`EncoderConfig`を参照してください．

音声を合成する際に渡す引数は以下の通りです：
- `inputText`：合成するテキスト
- `option`：合成される音声を調整するオプション．詳しくは，`SynthesisOption`を参照してください．

返り値の`stream`は`Readable`で，`encoder`設定の通りにエンコードされた音声データが流れます．合成はメインスレッドとは別のスレッドで行われます．
