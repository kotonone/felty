# felty

felty は、Web アプリケーションを実行可能ファイルに同梱し Windows / macOS / Linux 向けに配布するための Rust フレームワークです。

内部では Wry を使用しており、ビルドしたバイナリの最小サイズは Windows (x64) 向けビルドで 580KB 程度と超軽量になることが特徴です。
また、ポータブルかつ人間が理解しやすいディレクトリ構造を提供可能で、すべての挙動をカスタマイズ可能にすることに重点を置いています。

> [!WARNING]
> felty はまだ開発中のため、製品での使用は推奨しません。

> [!TIP]
> ファイルサイズを最小限にしたい、ブラックボックスな処理があることが許せないといった逸般ユーザー（主に私のことです）向けのフレームワークです。
> 単に Web アプリケーションを配布したいといった要望には、ほとんどの場合 Tauri が適しています。

## サンプル

```shell
cargo run -p felty-example
```

## Tips

### 軽量化を行う

felty を利用したアプリケーションを最小サイズにするには、Cargo.toml に以下のように記述してください。

なお、`trim-paths` は nightly でのみ使用可能です。

```toml
cargo-features = ["trim-paths"]

[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = true
trim-paths = true
```

また、felty の feature を無効化することでさらにサイズを削減することもできます。

## トラブルシューティング
### VCRUNTIME140.dll ライブラリが見つからない

Windows 環境において、felty で作成されたアプリケーションを起動する際に `VCRUNTIME140.dll` や `VCRUNTIME140_1.dll` が見つからないエラーが発生する場合があります。
これは、Microsoft Visual C++ 再頒布可能パッケージがインストールされていない環境で発生します。

PC に再頒布可能パッケージをインストールできない場合、`.cargo/config.toml` に以下のように記述することで、再頒布可能パッケージを静的リンクして同梱することができます。

ただし、MSVC ターゲットでのビルドに限るほか、ライブラリを同梱するとバイナリサイズが約 100 KB ほど増加します。

```toml
[target.i686-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```
