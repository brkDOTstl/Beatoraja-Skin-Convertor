# Beatoraja Skin Convertor

[日本語](./README.md) | **English**

An offline Rust/Tauri tool that converts LR2 skins into beatoraja-friendly copies. It supports
analysis, dry runs, individual conversion, and batch conversion without modifying the source skin.

## Current Status

The tool provides structural analysis, dry-run reports, non-destructive copy output, batch jobs,
and a bilingual Japanese/English desktop UI. It detects LR2 Play CSV files by content instead of
folder name and recognizes SD, HD, FHD, and 4K canvases.
Every `.dxa` archive is unpacked automatically during generation. This is a mandatory compatibility
stage because beatoraja cannot read LR2 fonts directly from DXA archives. The bundled
`DxaDecode.exe` runs locally, and the original `.dxa` files remain in the generated skin.
The LR2 real-time `num=114` POOR counter is also always mapped to beatoraja `num=426`
(POOR + MISS). This compatibility stage has no disable switch.
One-shot LR2 note bombs are also adjusted at their exact cycle boundary so beatoraja does not
briefly wrap them back to frame zero after the effect ends. Their animation speed is unchanged.

Implemented rules:

- `resolution-declaration`
- `theme-path-rebase`
- `asset-reference-repair`
- `follow-lift-op4`
- `key-laser-layer-compat`
- `judge-line-follow`
- `blend-fallback`
- `lift-cover-transparent`
- `lift-cover-copy-sudden`
- `green-number`
- `lift-number`
- `aspect-ratio-convert`
- `lr2-dark-mode`

Enabling `lr2-dark-mode` adds an `LR2 DARK MODE` customization entry to each converted Play skin,
with in-game `OFF`, `DARK`, and `DARK2` choices. Only CSV files recursively referenced by Play
headers are transformed; unrelated Result and Select modules are left unchanged.
Normal bombs, LN hold bombs, SUDDEN values, green numbers, and LIFT numbers remain visible in DARK2.
Native judgment lines and beatoraja compatibility images with matching source and destination
geometry remain visible in both DARK variants.
Both variants also preserve the groove gauge and helper images driven by gauge type, gauge-value
ranges, and gauge-rise or gauge-max timers.
This is an opt-in rule available in manual mode and is not enabled by the built-in presets.

Exposed but not automated yet:

- `lift-cover-custom-image`
- `key-laser-rewrite`
- `gauge-label-helper`

These three visual helpers require a skin-specific asset mapping and remain disabled in the UI.

## Prerequisites

Install Rust stable with Cargo. On Windows, Rust also needs the MSVC build tools and WebView2 used
by Tauri applications.

## CLI

Run from the workspace root:

```powershell
cargo run -p skin_cli -- analyze --input "D:\BMS Skin\Lr2\Askin" --json
cargo run -p skin_cli -- dry-run --input "D:\BMS Skin\Lr2\Askin" --preset transparent-lift --json
cargo run -p skin_cli -- generate --input "D:\BMS Skin\Skin Remod\Original Lr2\REMI-S (hachimi)" --output "D:\BMS Skin\Skin Remod\Generated\REMI-S-4x3" --preset fhd-4x3 --target-aspect 4:3
cargo run -p skin_cli -- generate --input "D:\BMS Skin\Lr2\Askin" --output "D:\BMS Skin\Skin Remod\Generated\Askin-FHD" --rule aspect-ratio-convert --rule resolution-declaration --resolution 2 --target-aspect 16:9 --preserve-bomb-aspect --preserve-nowjudge-aspect --preserve-nowcombo-aspect
```

An explicit `--resolution` automatically activates coordinate conversion and `#RESOLUTION`
declaration updates. An explicit `--target-aspect` automatically activates coordinate conversion;
the same dependency behavior applies to batch JSON and the desktop application.

For `#RESOLUTION,0` output, beatoraja horizontally stretches the 640x480 logical canvas to 16:9.
The converter therefore treats the selected layout aspect as the intended in-game aspect: 4:3
output is centered in a 480x480 logical area before rendering, while 16:9 output uses the full
640x480 logical canvas. This adjustment runs when a target layout aspect is selected.
Play-field frames that enclose a judgment area with equal left and right margins are rounded as a
single relationship, preventing fractional scaling from making one side one pixel narrower.

Batch job shape:

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

Run:

```powershell
cargo run -p skin_cli -- batch --job ".\batch.json" --json
```

## Desktop UI

The Tauri frontend is static HTML/CSS/JS under `apps/skin_generator_ui/ui`. Japanese and English
can be switched at runtime; the selection is stored locally. The batch queue snapshots each
skin's current paths, rules, LIFT strategy, target resolution/aspect, and three material-protection
options before sequential generation. Source skins and destination parent folders are selected
through native folder dialogs; the generated output is automatically named after the selected
source skin folder.

Run from the workspace:

```powershell
cargo run -p beatoraja_skin_convertor
```

The UI uses Tauri global `invoke`, so opening `index.html` directly only shows a static preview.
Build the optimized executable with `cargo build --release -p beatoraja_skin_convertor`. The output
is `target\\release\\beatoraja_skin_convertor.exe`.

## Verification Commands

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
