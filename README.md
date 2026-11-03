# MadeInRustNesEmuSample
「Made in Rust - NESエミュレータ」サンプルコード


## 概要

技術同人誌「Made in Rust - NESエミュレータ」内で記載したRustのソースコードをまとめたリポジトリです。本書では日本語によるファミコンエミュレータの包括的な解説を目指しています。

ここで提供するコードは筆者 `@furaibo` のオリジナルではなく、次の文献から引用の上、コード中のコメントや処理順序を組み替えたものとなります。APUに関して不足するコード部分についてはAI出力のコードも利用しています

- Writing NES Emulator in Rust
  - https://bugzmanov.github.io/nes_ebook/chapter_1.html
- ファミコンのAPUを実装してみよう
  - https://zenn.dev/pluswing/articles/f22f7f65179c06


## リポジトリの内容

リポジトリ内では「チェックポイント」として実装の区切りとなる部分のソースコード全体を掲載しています。

- checkpoint 1
  - CPU(公式命令のみ)が実装済みの状態です。
  - 動作させるとスネークゲームが起動しますが、キー操作はできません
- checkpoint 2-1
  - CPU(公式・非公式命令)/Bus/Cartridgeが実装済みの状態です。
  - 動作させるとスネークゲームが起動し、W/A/S/Dキーによる操作が可能です。
- checkpoint 2-2
  - CPU/Bus/Cartridgeが実装済みの状態です。
  - 動作させると `nestest` が起動し、CPU命令のテスト結果出力が可能です。
- checkpoint 3-1
  - CPU/Bus/Cartridge/PPU/Joypadが実装済みの状態です。
  - PPU: 固定画面の表示にのみ対応しています。
  - APU: 未実装のため音は出ません。
- checkpoint 3-2
  - CPU/Bus/Cartridge/PPU/Joypadが実装済みの状態です。
  - PPU: 固定画面+スクロール画面処理に対応しています。
  - APU: 未実装のため音は出ません。
- checkpoint 4
  - CPU/Bus/Cartridge/PPU/Joypad/APUが実装済み、本書における完成形の状態です。
  - PPU: 固定画面+スクロール画面処理に対応しています。
  - APU: 実装済みのため1〜5chの音が出せます。
  - エミュレータとしては最低限の実装のため、タイトルによっては動作しない可能性があります。


### "Write Nes Emulator in Rust" のソースコードとの対応

| チェックポイント       | GitHubリンク                                                              |
|----------------|------------------------------------------------------------------------|
| checkpoint 1   | TD                                                                     |
| checkpoint 2-1 | [ch5](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch5)     |
| checkpoint 2-2 | [ch5-1](https://github.com/bugzmanov/nes_ebook/tree/master/code/ch5.1) |
