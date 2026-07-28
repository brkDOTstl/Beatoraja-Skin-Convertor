# Beatoraja Skin Convertor

**日本語** | [English](./README.en.md)

LR2スキンをbeatoraja向けのコピーへ変換する、オフライン動作のRust/Tauri製ツールです。
元のスキンを変更せず、解析、ドライラン、個別変換、バッチ変換を行えます。

## 現在の機能

このツールは、構造解析、ドライランレポート、非破壊コピー出力、バッチジョブ、
日本語・英語を切り替えられるデスクトップUIを提供します。フォルダー名ではなく
CSVの内容からLR2 Play CSVを検出し、SD、HD、FHD、4Kのキャンバスを認識します。
生成時にはすべての`.dxa`アーカイブを自動的に展開します。これはオプションではなく、
beatorajaがLR2のDXA内フォントを直接読み込めない問題に対応する必須変換です。
内蔵の`DxaDecode.exe`をローカルで実行し、元の`.dxa`も出力側へ残します。

実装済みルール:

- `resolution-declaration`
- `theme-path-rebase`
- `asset-reference-repair`
- `follow-lift-op4`
- `judge-line-follow`
- `blend-fallback`
- `lift-cover-transparent`
- `lift-cover-copy-sudden`
- `green-number`
- `lift-number`
- `aspect-ratio-convert`
- `lr2-dark-mode`

`lr2-dark-mode`を有効にすると、変換後のPlayスキン設定に`LR2 DARK MODE`が追加され、
`OFF`、`DARK`、`DARK2`をゲーム側で選択できます。Playヘッダーから実際に参照されるCSVだけを
再帰的に変換し、他のResultやSelectモジュールは変更しません。
DARK2でもSUDDEN数値、green number、LIFT numberは表示したまま維持します。
この機能は手動モードで選択するオプトイン機能で、標準プリセットでは有効になりません。

UIには表示されますが、まだ自動化されていないルール:

- `lift-cover-custom-image`
- `key-laser-rewrite`
- `gauge-label-helper`

この3つの表示補助にはスキン固有の素材マッピングが必要なため、現在はUIで無効化されています。

## 必要環境

Cargoを含むRust stableをインストールしてください。Windowsでは、RustのMSVCビルドツールと
Tauriアプリケーションが使用するWebView2も必要です。

## CLI

ワークスペースのルートで実行します。

```powershell
cargo run -p skin_cli -- analyze --input "D:\BMS Skin\Lr2\Askin" --json
cargo run -p skin_cli -- dry-run --input "D:\BMS Skin\Lr2\Askin" --preset transparent-lift --json
cargo run -p skin_cli -- generate --input "D:\BMS Skin\Skin Remod\Original Lr2\REMI-S (hachimi)" --output "D:\BMS Skin\Skin Remod\Generated\REMI-S-4x3" --preset fhd-4x3 --target-aspect 4:3
cargo run -p skin_cli -- generate --input "D:\BMS Skin\Lr2\Askin" --output "D:\BMS Skin\Skin Remod\Generated\Askin-FHD" --rule aspect-ratio-convert --rule resolution-declaration --resolution 2 --target-aspect 16:9 --preserve-bomb-aspect --preserve-nowjudge-aspect --preserve-nowcombo-aspect
```

`--resolution`を明示すると、座標変換と`#RESOLUTION`宣言の更新が自動的に有効になります。
`--target-aspect`を明示した場合も座標変換が自動的に有効になります。バッチJSONと
デスクトップアプリケーションでも同じ依存関係が適用されます。

`#RESOLUTION,0`では、beatorajaが640x480の論理キャンバスを16:9へ横方向に引き伸ばします。
そのため、選択したレイアウト比率をゲーム内で意図する比率として扱います。4:3出力は
描画前に480x480の論理領域へ中央配置し、16:9出力は640x480の論理キャンバス全体を使用します。
この調整はターゲットのレイアウト比率を選択した場合に実行されます。
判定領域を左右同じ余白で囲むプレイフィールド外枠は、その包含関係を検出して余白を一組として
丸めるため、端数スケーリングによる左右1pxの片寄りを防ぎます。

バッチジョブの形式:

```json
{
  "entries": [
    {
      "input": "D:\\BMS Skin\\Lr2\\Askin",
      "output": "D:\\BMS Skin\\Skin Remod\\Generated\\Askin",
      "preset": "visible-lift",
      "resolution": null,
      "target_aspect": null,
      "aspect_protection": {
        "bomb_effects": true,
        "nowjudge": true,
        "nowcombo": true
      },
      "lift_cover_strategy": "copy-sudden",
      "rules": []
    }
  ]
}
```

実行コマンド:

```powershell
cargo run -p skin_cli -- batch --job ".\batch.json" --json
```

## デスクトップUI

Tauriフロントエンドは`apps/skin_generator_ui/ui`内の静的HTML/CSS/JSで構成されています。
実行中に日本語と英語を切り替えられ、選択した言語はローカルに保存されます。バッチキューへ
追加した時点で、各スキンのパス、ルール、LIFT方式、ターゲット解像度・比率、3種類の
素材比率保護オプションを保存し、順番に生成します。変換元スキンと出力先の親フォルダーは
ネイティブのフォルダー選択ダイアログで指定し、出力フォルダー名は変換元フォルダー名から
自動生成されます。

ワークスペースから実行:

```powershell
cargo run -p beatoraja_skin_convertor
```

UIはTauriのグローバル`invoke`を使用するため、`index.html`を直接開いた場合は静的プレビューのみ
表示されます。最適化済み実行ファイルは`cargo build --release -p beatoraja_skin_convertor`で
ビルドできます。出力先は`target\\release\\beatoraja_skin_convertor.exe`です。

## 検証コマンド

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
node --check apps\skin_generator_ui\ui\main.js
node --check apps\skin_generator_ui\ui\batch.js
node --check apps\skin_generator_ui\ui\render.js
node --check apps\skin_generator_ui\ui\i18n.js
```
