# grid-images

複数の画像を中央で切り抜き、同じサイズのセルを基準にグリッド状に並べる CLI ツール

## インストール

ソースからビルド:

```sh
cargo install --git https://github.com/yuma140902/grid-images
```

## 使い方

```sh
grid-images photo1.jpg photo2.png photo3.webp -o grid.png
```

列数と各タイルのサイズも指定できる。

```sh
grid-images images/*.jpg --columns 3 --tile-size 400 --output result.png
```

- 入力順に左上から配置する。
- 縦横比が 1.7:1 以上の横長画像は 2:1、1:1.7 以上の縦長画像は 1:2 に切り抜き、2 セル分を使って配置する。それ以外は正方形に切り抜く。
- `--columns` を省略すると、全体が正方形に近くなる列数を選ぶ。
- 最終行の余った領域は透過になる (出力形式が対応している場合)。
- 出力形式は出力ファイルの拡張子から判定する。
