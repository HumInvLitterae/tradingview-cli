# `tv` はじめての使い方

この文書は、GitHub Releases から `tv` の配布アーカイブをダウンロードした
ユーザーと、その `tv` を使うエージェント AI 向けの案内です。開発者向けの
ビルド手順ではありません。

`tv` は TradingView 用のコマンドラインツールです。TradingView Desktop を
開かずにデータを読むコマンドと、手元の TradingView Desktop に接続して
チャートや画面状態を読むコマンドがあります。`tv` は TradingView のアカウント、
サブスクリプション、ペイウォール、取引所データ契約、スクリプト所有権を
回避するものではありません。

## 1. 配布アーカイブをダウンロードする

GitHub Releases から、自分の環境に合うファイルをダウンロードします。

- Windows: `tv-<tag>-x86_64-pc-windows-msvc.zip`
- macOS Apple Silicon: `tv-<tag>-aarch64-apple-darwin.tar.gz`
- macOS Intel: `tv-<tag>-x86_64-apple-darwin.tar.gz`
- Linux: `tv-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- チェックサム: `SHA256SUMS`

可能であれば `SHA256SUMS` でアーカイブを確認してから展開してください。
アーカイブには、`tv` 本体、README、CHANGELOG、ライセンス、この導入文書、
エージェント向けガイド、実行時用のスキルが含まれます。

## 2. `tv` を実行できる場所に置く

展開したディレクトリから直接実行できます。

- macOS/Linux: `./tv --version`
- Windows PowerShell: `.\tv.exe --version`

普段使いする場合は、実行ファイルを `PATH` が通った場所に置くと、ユーザーも
エージェント AI も `tv ...` として実行できます。

```bash
tv --version
```

期待したバージョンが表示されれば、まずは `tv` を実行できる状態です。

## 3. エージェント AI と一緒に使う

配布アーカイブには、エージェント向けの `AGENTS.md`、`CLAUDE.md`、
`.agents/skills/`、`.claude/skills/` が含まれます。ローカルファイルを読めて、
シェルコマンドを実行できるエージェントアプリに、これらのファイルを読ませて
ください。

起動方法は、使うアプリによって少し変わります。

- `tv` を `PATH` が通った場所に置いた場合、エージェントはどの作業フォルダから
  でも `tv ...` と実行できます。
- `tv` を `PATH` に置かない場合は、エージェントのカレントフォルダを配布
  アーカイブを展開したフォルダに合わせ、macOS/Linux では `./tv ...`、
  Windows では `.\tv.exe ...` と実行させます。
- エージェントが別のプロジェクトのフォルダで作業している場合は、`tv` を
  `PATH` に置くか、展開した実行ファイルの場所をエージェントに渡してください。

最初の依頼は、たとえば次のようにします。

> 同梱されている `tv` を使ってください。最初に `tv --version` を実行して
> ください。TradingView Desktop を使わずに読める情報で足りる場合は、
> そちらを優先してください。Desktop のチャートを読む前には `tv readiness`
> を確認してください。実行したコマンドと、どの種類の情報を読んだのかを
> 報告してください。TradingView の状態を変更する操作は、事前に確認を
> 取ってください。

大事なのは、似た名前のコマンドでも「読んでいるもの」が違うことです。たとえば、
`tv bars` は TradingView Desktop を使わずに履歴の足を取得します。一方で
`tv ohlcv` は、いま選択されている Desktop チャートの足を読みます。どちらも
便利ですが、同じ根拠として扱うと検証結果がずれます。

最初に覚えるとよい使い分けは次のとおりです。

- `tv quote`、`tv quotes`、scanner、fundamentals、`tv bars` は
  TradingView Desktop を使わずに読み取ります。
- `tv bars` は、再現可能な履歴の足を取得するための入口です。
- `tv range` は、表示中の Desktop チャートの表示範囲を動かすだけです。
- `tv ohlcv` は、選択中の Desktop チャートから足を読みます。
- `tv quote --source quote-data` は、Desktop を使って quote-data という
  TradingView 内部の価格情報を明示的に読みます。
- `tv observe chart` と `tv stream ...` は、選択中の Desktop チャートを
  一定時間だけ観測し、1行ずつ JSON を出します。

エージェントには、実行したコマンドと、どの種類の情報を読んだのかを報告させて
ください。履歴データ、選択中チャート、価格情報を混同しないようにするためです。
`tv` は市場データをランキング、スコア、売買推奨、投資助言に変換するツールでは
ありません。

## 4. TradingView Desktop なしで動作確認する

最初は TradingView Desktop を使わない読み取りコマンドで確認するのが安全です。

```bash
tv quote AAPL
tv info NASDAQ:AAPL
tv bars NASDAQ:AAPL --timeframe 1D --count 5
```

古いチャート例や検証用の履歴データを取得したい場合は、表示中チャートを動かす
のではなく `tv bars` を使います。

```bash
tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:CRUS --timeframe 1W --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:CRUS --timeframe 1M --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:AAPL --timeframe 60 --from 2026-05-01 --to 2026-05-22 --count 1000
```

日付範囲を指定した場合は、まず `range_coverage_status` と
`range_alignment` を確認してください。日付範囲指定で使える時間軸は、現時点
では 15 分足、60 分足、日足、週足、月足です。そのほかの分足は、日付範囲指定
ではまだ使えません。分足、週足、月足では、足の時刻はその期間の開始時刻や
開始日を表します。指定した開始日から終了日までの範囲に、その時刻が入っている
足だけが返ります。日付範囲を指定した場合の `--count` は返す足の最大本数で、
指定しなければ 500 本、最大で 5000 本です。直近本数を取る通常の使い方では、
最大 500 本のままです。さらに `range_fetch_summary` を見ると、追加取得を何回
行ったか、返却本数の上限で切られたか、取得元や待ち時間の都合で範囲を満たせな
かったかを確認できます。

## 5. TradingView Desktop を使う準備をする

チャート状態、スクリーンショット、Pine、Replay、Screener などを扱うには、
手元の TradingView Desktop セッションが必要です。この準備は、普通にアプリを
開くだけとは少し違います。`tv` が TradingView Desktop と通信できるように、
ローカル接続用の設定付きで起動されている必要があります。

一番簡単なのは、`tv` に TradingView Desktop の起動または既存セッションの再利用を
任せる方法です。

```bash
tv launch
tv readiness
tv tab list
tv state
```

`tv launch` は、まず既に接続できる TradingView Desktop があるかを確認します。
接続できる場合は、そのまま既存のセッションを使います。接続できない場合は、`tv`
が必要とするローカル接続用の設定を付けて TradingView Desktop を起動しようと
します。その後、`tv readiness` でチャートを読める状態かを確認し、`tv tab list`
で接続先の一覧を見て、`tv state` で選択中チャートを読めることを確認します。

`tv launch` が TradingView Desktop を見つけられない場合は、実行ファイルの
場所を指定します。

```bash
tv launch --path <TRADINGVIEW_DESKTOP_PATH>
```

エージェント AI に任せる場合は、次のように依頼します。

> `tv launch` を実行して、`tv` が使える形で TradingView Desktop を起動または
> 再利用してください。その後、`tv readiness`、`tv tab list`、`tv state` を
> 実行して結果を報告してください。`tv launch` がアプリを見つけられない場合は、
> TradingView Desktop の場所を私に確認してください。この準備中に、チャートの
> 銘柄、時間足、アラート、描画、アカウント状態は変更しないでください。

手動で TradingView Desktop を先に開いてから、エージェントに `tv readiness` を
実行させる方法もあります。ただし、それで接続できない場合は、`tv launch` を使って
`tv` が必要とするローカル接続用の設定付きで起動してください。

複数の TradingView 接続先が開いている場合は、`tv tab list` や
`tv readiness` に表示される `target_cli_args` を使います。

```bash
tv --target-id <ID> state
```

実際の接続先 ID、つまり `--target-id` に渡す値は、手元のセッションに紐づく
情報です。接続先 ID、アカウント固有の ID、クッキー、トークン、ローカルの
パスを共有メモや公開文書に貼らないでください。

## 6. 操作より先に読み取りで確認する

チャート、アカウント、Pine、Replay、Screener、アラート、ウォッチリスト、
描画、レイアウトを変更する前に、まず読み取りコマンドで状態を確認します。

```bash
tv state
tv ohlcv --summary --count 100
tv screenshot --region chart --output target/tv-chart.png
```

短時間だけ選択中チャートを観測したい場合は、1行ずつ JSON を出す観測コマンドを
使います。

```bash
tv observe chart --duration-ms 10000 --heartbeat-ms 2000
tv stream bars --max-events 5
```

これらのコマンドは、準備状態、サンプル、定期的な状態通知、最後のまとめを出します。
選択中の Desktop チャートを観測するものであり、TradingView Desktop を使わない
履歴の足の取得や、複数銘柄のリアルタイム配信ではありません。

## 7. 次に読むもの

- `README.md`: プロジェクト概要と主要コマンド例。
- `AGENTS.md` / `CLAUDE.md`: 配布アーカイブ内でエージェントに読ませる実行時ガイド。
- `docs/command-source-taxonomy.md`: リポジトリ内の詳しいコマンド分類。
- `docs/observation-workflows.md`: リポジトリ内の実用的な読み取り手順。
