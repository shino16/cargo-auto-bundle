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
  * `<entry-point>` ファイルの中身が先に出力されます。`<crate>` の中で定義されたマクロを `<entry-point>` で使う場合には注意してください。
  * ファイル構造は `(公開性) mod (モジュール名) { ... }` という形で反映されます。
  * 該当ファイル中の `use crate::...` は `use crate::(クレート名)::...` で置き換えられます。
  * 該当ファイル中の `(公開性) mod (モジュール名);` は削除されます。
  * `<entry-point>` 中の `use (クレート名)::...` は `use crate::(クレート名)::...` で置き換えられます。

## 注意

1. **相対パスに対応していません。** 対象クレート内の要素に対して `use` 宣言を行うとき、`<entry-point>` 内では `use <crate>::...` 、`<crate>` 内では `use crate::...` のように書いてください。
2. `use` 宣言されていないモジュールは、使われていたとしても走査対象になりません。例えば `let inv = my_library::math::modpow(n, MOD - 2, MOD);` のような記述があっても、`use my_library::math;` のような記述がなければ `math` モジュールは出力に反映されません。
3. `use` 宣言以外の場所で `crate::...` と書かれていても、`crate::(クレート名)` には置き換えられません。
4. モジュールの `path` 属性は考慮しません。
5. 仕組み上 `mod a { pub mod b; }` というような記述があった場合に `mod a` が2回定義されます。
6. `<crate>` はライブラリクレートであることが仮定されています。
7. 手続きマクロの展開は一切行いません。
8. `pub(crate)` や `pub(in (パス))` に対して特別な処理は行っていません。
9. 動作の正当性は保証しません。本プログラムの不適切な動作によって発生したペナルティについて責任を負いません。

これらのうち1.–4.はそのうち実装するかもしれません。バイナリクレートも対応したほうが良いですか？
7.の前半はさすがに厳しいです。あとは、うーん…

作者本人がまだあまり使っていないので、バグを含んでいても気づいていない可能性があります。何かあれば連絡いただければ幸いです。
