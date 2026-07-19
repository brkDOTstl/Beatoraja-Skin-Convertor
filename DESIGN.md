# Beatoraja Skin Convertor Design

## Product Shape

This is a local conversion workbench for LR2 skins. The first screen is the work surface:
source path, output path, preset, analysis, rule toggles, dry-run, and generate. It is not a
landing page.

## User

The user has many legacy skins and needs repeatable local conversion without an LLM. They still
need judgment per skin, so the UI must expose uncertain modules instead of hiding them behind one
automatic button.

## Layout

- Left column: paths, preset, conversion options, and command buttons.
- Center: analysis summary and rule toggles grouped by module.
- Right column: warnings, changed files, manifest preview, and local help text.

## Visual Direction

Quiet utility UI with dense information. Use restrained neutrals, clear section headers, stable
button sizes, and compact form controls. Avoid decorative hero sections, large cards, gradients,
and marketing copy.

Core UI tokens live in `styles.css`: every rendered surface/state color, spacing step, control and
row dimension, border/focus width, radius, type size/weight/line height, desktop column, and output
height is named there. The 1080 px and 760 px media-query thresholds remain literal because CSS
custom properties cannot drive media conditions. Repeated rows are the only card-like elements.

## Interaction Rules

- Source folders are never modified.
- Generate requires an output folder distinct from the input folder.
- Dry-run is available before generate.
- Risky rules are opt-in and keep their warning visible after selection.
- LIFT cover, green number, and LIFT number are separate toggles.
- Unsupported rules show why they are skipped.
- English and Japanese can be switched without restarting; the preference persists locally.
- Analysis reports the detected native resolution, canvas, aspect ratio, and evidence source.
- Target resolution and target aspect are independent: users may upscale or downscale coordinates,
  keep the detected aspect, or deliberately stretch/compress the layout between 4:3 and 16:9.
- Bomb/LN/explosion effects, NOWJUDGE, and NOWCOMBO each have an independent aspect-protection
  checkbox. Protected objects follow converted layout positions without non-uniform image scaling.
- Aspect protection preserves the source skin's existing shape; it never attempts to repair artwork
  or geometry mistakes made by the skin author.
- Each batch queue item snapshots the current paths, options, and selected rules so different skin
  families can be configured independently before one sequential generation run.
