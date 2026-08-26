# grid-images

複数の画像を中央で正方形に切り抜き、同じサイズに揃えてグリッド状に並べる CLI ツールです。

## インストール

```sh
cargo install --path .
```

## 使い方

```sh
grid-images photo1.jpg photo2.png photo3.webp -o grid.png
```

列数と各タイルのサイズも指定できます。

```sh
grid-images images/*.jpg --columns 3 --tile-size 400 --output result.png
```

- 入力順に左上から配置します。
- `--columns` を省略すると、全体が正方形に近くなる列数を選びます。
- 最終行の余った領域は透過になります（透過を保持するには PNG などを推奨します）。
- 出力形式は出力ファイルの拡張子から判定します。

詳しいオプションは `grid-images --help` で確認できます。

## 開発

```sh
cargo test
```

