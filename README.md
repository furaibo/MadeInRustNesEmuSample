# MadeInRustNesEmuSample

## 概要

技術同人誌「Made in Rust - NESエミュレータ」内で記載したRustのソースコードをまとめたリポジトリです。本書ではRustによるファミコンエミュレータ実装の詳細な解説を目指しています。ただし本書でのエミュレータは最低限動作可能な実装であるため、タイトルによっては正しく動作しない可能性がある点はご容赦ください。

ここで提供するコードは筆者 `@furaibo` のオリジナルではなく、次の文献から引用の上、コメントを追記し処理順序を組み替えたものとなります。一部のAPU機能の実装やコードのデバッグのためAIの出力も利用しています。

### 参考文献

- Writing NES Emulator in Rust
  - https://bugzmanov.github.io/nes_ebook/chapter_1.html
- ファミコンのAPUを実装してみよう
  - https://zenn.dev/pluswing/articles/f22f7f65179c06

### 参考リポジトリ

- bugzmanov/nes_ebook
  - https://github.com/bugzmanov/nes_ebook}
- pluswing - ファミコンエミュレーターを作る！
  - https://github.com/pluswing/famicon_emulator}



## コードの内容

このリポジトリでは「チェックポイント」として実装の区切りとなる部分のソースコード全体を掲載しています。

### 各チェックポイントについての概要

- checkpoint 1
  - CPU(公式命令のみ)が実装済みの状態です。
  - 動作させるとスネークゲームが起動し、W/A/S/Dキーによる操作が可能です。
  - スネークゲームはmain関数内にハードコードされた状態です。
- checkpoint 2-1
  - CPU/Bus/Cartridgeが実装済みの状態です。
  - 動作させるとスネークゲームが起動し、W/A/S/Dキーによる操作が可能です。
  - スネークゲームはnesファイル化された状態です。
- checkpoint 2-2
  - CPU/Bus/Cartridgeが実装済みの状態です。
  - 動作させると `nestest.nes` が起動し、CPU命令テスト結果の出力が可能です。
- checkpoint 3-1
  - CPU/Bus/Cartridge/PPU/Joypadが実装済みの状態です。
  - PPU: 固定画面の表示のみ対応しています(表示バグあり)。
  - APU: 未実装のため音は出ません。
- checkpoint 3-2
  - CPU/Bus/Cartridge/PPU/Joypadが実装済みの状態です。
  - PPU: 固定画面+スクロール画面処理に対応しています(表示バグあり)。
  - APU: 未実装のため音は出ません。
- checkpoint 4
  - CPU/Bus/Cartridge/PPU/Joypad/APUが実装済みの状態です。
  - PPU: 固定画面+スクロール画面処理に対応しています(表示バグあり)。
  - APU: 1〜5chの音が出ます(音声バグあり)。
- checkpoint 5
  - CPU/Bus/Cartridge/PPU/Joypad/APUが実装済みの状態です。
  - PPU/APUとも不具合・バグ修正済み、スーパーマリオブラザーズが動作する状態です。


### 実行方法

基本的には各checkpointのディレクトリへと入り、`cargo run`コマンドを実行することで処理を実行できます。

PPUの実装が入った "checkpoint3-1" 以降については、コマンドライン引数の指定による実行ファイルの指定も可能です。
もしも実行ファイルを指定しない場合には `rom/color_test.nes` がデフォルトで実行されるようになっています。
動作確認にあたり、パックマンやスーパーマリオブラザーズといったゲームのROMファイルにつきましては各自でご準備ください。

ROMファイルへのパスを指定する場合は以下のように記述します。

```
$ cargo run -- -f rom/pacman.nes
```


### 本書の各章との対応

| チェックポイント       | 対応する章                                         |
|----------------|-----------------------------------------------|
| checkpoint 1   | 第2章 エミュレータの実装 ― CPU編                          |
| checkpoint 2-1 | 第3章 エミュレータの実装 ― バス・カートリッジ・ROMテスト編 (ROMテスト前まで) |
| checkpoint 2-2 | 第3章 エミュレータの実装 ― バス・カートリッジ・ROMテスト編 (ROMテスト後)   |
| checkpoint 3-1 | 第4章 エミュレータの実装 ― PPU編(1) (固定画面処理)              |
| checkpoint 3-2 | 第5章 エミュレータの実装 ― PPU編(2) (スクロール画面処理)           |
| checkpoint 4   | 第6章 エミュレータの実装 ― APU編 (音声処理)                   |
| checkpoint 5   | 対応なし(※PPU/APUバグ修正、スーパーマリオブラザーズ動作確認版)          |



### "Writing Nes Emulator in Rust" のソースコードとの対応

| チェックポイント       | 対応リンク                                                            |
|----------------|------------------------------------------------------------------|
| checkpoint 1   | [ch3.4](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch3.4) |
| checkpoint 2-1 | [ch5](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch5) |
| checkpoint 2-2 | [ch5.1](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch5.1) |
| checkpoint 3-1 | [ch6.4](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch6.4) |
| checkpoint 3-2 | [ch8](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch8) |
| checkpoint 4   | 対応なし                                                             |
| checkpoint 5   | 対応なし                                                             |

