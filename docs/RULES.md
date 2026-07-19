# Rule Notes

Core beatoraja constants:

- `num=313`: beatoraja green number / duration.
- `num=314`: beatoraja LIFT number.
- `op2=271`: lanecover enabled.
- `op2=272`: LIFT enabled.
- `op4=3`: shift a destination upward by the LIFT/lanecover amount.
- `op4=4`: green-number special placement used by known ports.

Implemented rules:

- `resolution-declaration`: detect and insert a missing `#RESOLUTION`, or replace it when an explicit
  target resolution is selected.
- `theme-path-rebase`: rewrite source-theme references in lr2skin and CSV records.
- `asset-reference-repair`: clear missing thumbnails, remove unused missing fonts, and repair movie
  extensions while recognizing assets packed in DXA archives.
- `follow-lift-op4`: apply `op4=3` to complete bomb, LN, tall laser, and rhythm-pulse source groups.
- `judge-line-follow`: apply `op4=3` to every judge-line animation frame.
- `lift-cover-copy-sudden`: pair SUDDEN sliders with judgment sides and start the LIFT drawing at
  the lower edge of the original judgment line.
- `lift-cover-transparent`: add functional invisible LIFT objects for sparse play-field layouts.
- `green-number` and `lift-number`: clone complete lane-cover number animation groups as `num=313`
  and `num=314`.
- `aspect-ratio-convert`: scale `x/y/w/h` coordinates from the detected source canvas to the selected
  SD, HD, FHD, or 4K canvas and fit either a 4:3 or 16:9 layout. If no target aspect is supplied, the
  detected source aspect is retained. Bomb/LN/explosion groups, `NOWJUDGE`, and `NOWCOMBO` can each
  preserve their source width-to-height ratio while their centers follow the converted layout. The
  rule uses command-specific LR2/beatoraja field ownership; unsupported destination commands remain
  unchanged instead of being treated as generic rectangles.
  For `#RESOLUTION,0` output, 4:3 uses a centered 480x480 logical area to counter beatoraja's
  horizontal stretch, while 16:9 keeps the full 640x480 logical canvas unchanged.

Explicit target resolution selects both `resolution-declaration` and `aspect-ratio-convert` in every
entry point. Explicit target aspect selects `aspect-ratio-convert`, including CLI and batch jobs.

Aspect protection only prevents new non-uniform scaling. It deliberately does not inspect or repair
artwork and geometry mistakes already present in the source skin.
- `blend-fallback`: replace LR2-only destination `blend=5` with beatoraja-compatible `blend=2`, and disable structurally matched duplicate option-label fades that darken in beatoraja.

Key-laser sprite reconstruction, gauge-label sprite ordering, and arbitrary custom LIFT images stay
manual because their source rectangles and animation ordering are skin-specific.
