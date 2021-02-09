# cargo-auto-bundle

競技プログラミングで必要なライブラリのコピペ作業を補助するツールです。

## インストール

```
cargo install --git https://github.com/shino16/cargo-auto-bundle
```

## 使い方

```
cargo auto-bundle [--crate <crate [default: .]>] [--entry-point <entry-point [default: src/main.rs]>] [--list-deps]
```

* `<crate>/Cargo.toml` をパースし、クレート名を取得します。
* `<entry-point>` ファイルを起点に対象クレートの要素（モジュール／その他）に対する `use` 宣言を辿り、依存するファイルを列挙します。
* `--list-deps` が渡されたとき、これらのファイルへのパスを一行ずつ出力します。これは [`online-judge-tools/verification-helper`](https://github.com/online-judge-tools/verification-helper) と一緒に使えます。例：[`.github/workflows/ci.yml`](https://github.com/shino16/cpr/blob/master/.github/workflows/ci.yml) [`.verify-helper/config.toml`](https://github.com/shino16/cpr/blob/master/.verify-helper/config.toml)
* そうでない場合は、これらのファイルを `<entry-point>` ファイルとまとめて出力します。このとき、
  * `<entry-point>` ファイルの中身が先に出力されます。
  * ファイル構造は `(公開性) mod (モジュール名) { ... }` という形で反映されます。
  * 該当ファイル中の `use crate::...` は `use crate::(クレート名)::...` で置き換えられます。（マクロ中を除く）
    * `#[macro_export]` 属性が付されたマクロを
  * 該当ファイル中の `(公開性) mod (モジュール名);` は削除されます。
  * `<entry-point>` 中の `use (クレート名)::...` は `use crate::(クレート名)::...` で置き換えられます。（マクロ中を除く）

## 注意

1. ~~**相対パスに対応していません。** 対象クレート内の要素に対して `use` 宣言を行うとき、`<entry-point>` 内では `use <crate>::...` 、`<crate>` 内では `use crate::...` のように書いてください。~~ とりあえず基本的な使い方については対応したつもりです
2. `use` 宣言されていないモジュールは、使われていたとしても走査対象になりません。例えば `let inv = my_library::math::modpow(n, MOD - 2, MOD);` のような記述があっても、`use my_library::math;` のような記述がなければ `math` モジュールは出力に反映されません。
3. `use` 宣言以外の場所で `crate::...` と書かれていても、`crate::(クレート名)` には置き換えられません。（`use` 宣言は `rust-analyzer` が勝手に入れてくれるので、さぼっています）
4. `(公開性) mod XXX;` に付された属性は無視され、出力に含まれません。
5. 仕組み上 `pub mod a { pub mod b; }` というような記述があった場合に、この記述と `pub mod a { pub mod b { ... } }` で `mod a` が2回定義されます。
6. `<crate>` はライブラリクレートであることが仮定されています。
7. 手続きマクロの展開は一切行いません。
8. 動作の正当性は保証しません。本プログラムの不適切な動作によって発生したペナルティについて責任を負いません。
