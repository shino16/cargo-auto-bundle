# cargo-auto-bundle

Rustライブラリのコピペ作業やライブラリ管理を補助する競プロ向けツールです。

## インストール

```bash
$ cargo install --git https://github.com/shino16/cargo-auto-bundle
```

## 使い方

```bash
$ cargo auto-bundle [--crate <crate [default: .]>] [--entry-point <entry-point [default: src/main.rs]>] [--list-deps]
```

## 使用例

`lib`（クレート名）は任意のクレート名で置き換えてください。

```rust
use lib::ds::fenwick::*;
use proconio::*;

#[fastout]
fn main() {
    input! {
        n: usize, q: usize,
        a: [u32; n],
        txy: [(u32, usize, usize); q],
    }
    let mut fwk = FenwickTree::new(a, GroupImpl(|| 0, |a, b| a ^ b, |a| a));
    for (t, x, y) in txy {
        match t {
            1 => fwk.add(x - 1, y as u32),
            _ => println!("{}", fwk.ask(x - 1, y)),
        }
    }
}

```

このコードを `lib` クレート中の `src/main.rs` に置き、次を実行します：

```bash
$ cargo auto-bundle > tmp.rs
```

これを提出⇒[AC](https://atcoder.jp/contests/abc185/submissions/20195269)

このコードが依存する `lib::ds::fenwick` モジュールと、`lib::ds::fenwick` が依存する `lib::alg`、`lib::bits` の中身がモジュール構造を保って展開されています。

また、これは [`online-judge-tools/verification-helper`](https://github.com/online-judge-tools/verification-helper) と組合せて使うことができます。

例：[`.github/workflows/ci.yml`](https://github.com/shino16/cpr/blob/master/.github/workflows/ci.yml) / [`.verify-helper/config.toml`](https://github.com/shino16/cpr/blob/master/.verify-helper/config.toml)

なお、[`online-judge-tools/verification-helper`](https://github.com/online-judge-tools/verification-helper) には [Rustサポートが追加された](https://github.com/online-judge-tools/verification-helper/pull/346) ので、少し [書きかえ](https://github.com/shino16/verification-helper/commit/ac15e8072a522833c4dad69fa1414edd23beade9) が必要です。

## 詳細

* `<crate>/Cargo.toml` をパースし、クレート名を取得します。
* `<entry-point>` ファイルを起点に、対象クレート内のモジュールや構造体・トレイト等に対する `use` 宣言を辿り、依存するファイルを列挙します。
* `--list-deps` が渡されたとき、これらのファイルへのパスを一行ずつ出力します。
* そうでない場合は、これらのファイルを `<entry-point>` ファイルとまとめて出力します。このとき、
  * `<entry-point>` ファイルの中身が先に出力されます。
  * ファイル構造は `(公開性) mod (モジュール名) { ... }` という形で反映されます。
  * `<entry-point>` ファイル中の `use (クレート名)::...` と `<crate>` 内のファイルの `use crate::...` は、ともに `use crate::(クレート名)::...` で置き換えられます。（マクロの中身を除く）
    * `#[macro_export]` 属性が付された単一のマクロに対する `use` 宣言に対しては、特別な処理を行います。`use` 宣言のパスが `crate::` から始まる場合は何の処理も行わず、`(クレート名)::` から始まる場合は宣言ごと削除します。
    * 例えばモジュール `a` 内でマクロ `x` を `#[macro_export]` 付きで定義した場合、モジュール `a` 内に `pub use crate::x;` と書いて、これを使うときは `use (クレート名)::a::*;` とするよいと思います。
    * より簡単なのは、クレートのトップレベル（`<crate>/lib.rs`）でマクロを定義することです。これを直接 `use (クレート名)::x;` とすればうまくいきます。
  * `<crate>` 中の `(pub) mod (モジュール名);` は削除されます。

## 注意

1. ~~**相対パスに対応していません。** 対象クレート内の要素に対して `use` 宣言を行うとき、`<entry-point>` 内では `use <crate>::...` 、`<crate>` 内では `use crate::...` のように書いてください。~~ とりあえず基本的な使い方については対応したつもりです
1. `use` 宣言されていないモジュールは、使われていたとしても走査対象になりません。例えば `let inv = my_library::math::modpow(n, MOD - 2, MOD);` のような記述があっても、`use my_library::math;` のような記述がなければ `math` モジュールは出力に反映されません。
1. `use` 宣言以外の場所で `crate::...` と書かれていても、`crate::(クレート名)` には置き換えられません。（`use` 宣言は `rust-analyzer` が勝手に入れてくれるので、さぼっています）
1. `<crate>` がバイナリクレートであった場合の動作は未確認です。
1. `use` されていないモジュールの `impl` やマクロ定義は補足できません。
1. マクロ定義・呼び出しの中身に含まれる `use` 宣言は無視されます。
1. `(公開性) mod XXX;` に付された属性は無視され、出力に含まれません。
1. 手続きマクロの展開は一切行いません。
1. [自分のライブラリ](https://github.com/shino16/cpr) でそれっぽく動くことしか確認していません。本プログラムの動作に関連して発生したいかなる結果（CE、WA、TLE、…）について責任を負いません。
