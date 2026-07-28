# Rule Notes

Core beatoraja constants:

- `num=313`: beatoraja green number / duration.
- `num=314`: beatoraja LIFT number.
- `op2=271`: lanecover enabled.
- `op2=272`: LIFT enabled.
- `op4=3`: shift a destination upward by the LIFT/lanecover amount.
- `op4=4`: green-number special placement used by known ports.

Mandatory conversion stage:

- Every `.dxa` archive is unpacked after the source skin is copied and before compatibility rules
  run. This makes LR2 fonts available as ordinary `font.lr2font` and image files for beatoraja.
  The embedded LR2 `DxaDecode.exe` runs offline, the source archive remains in the output, and all
  extracted files are recorded in the output manifest. This stage has no disable switch.
- Every LR2 real-time `#SRC_NUMBER` POOR counter (`num=114`) is mapped to beatoraja
  `num=426`, whose value is POOR + MISS. This restores LR2's combined normal/empty-POOR count and
  is applied before optional rules, including DARK variant generation. This stage has no disable
  switch.

Implemented rules:

- `resolution-declaration`: detect and insert a missing `#RESOLUTION`, or replace it when an explicit
  target resolution is selected.
- `theme-path-rebase`: rewrite source-theme references in lr2skin and CSV records.
- `asset-reference-repair`: clear missing thumbnails, remove unused missing fonts, and repair movie
  extensions while recognizing assets packed in DXA archives.
- `follow-lift-op4`: apply `op4=3` to complete bomb, LN, tall laser, and rhythm-pulse source groups.
- `key-laser-layer-compat`: preserve LR2's visual layering in beatoraja by moving complete tall
  key-laser groups before the first note-renderer registration point. Pure key-laser `#IF` blocks
  move intact; mixed conditional blocks are reported and left unchanged.
- `judge-line-follow`: materialize LR2's solid black/white references (`gr=110/111`) as standalone
  1x1 PNG assets because beatoraja's dedicated judge-line loader does not resolve those references.
  Keep native `SRC_JUDGELINE` / `DST_JUDGELINE` rendering and original geometry, then apply `op4=3`
  to every frame. Other system references retain the normal-image compatibility fallback.
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
  When a play-field rectangle encloses a judgment area with equal left and right margins, those
  margins are scaled and rounded as one relationship. This prevents an unavoidable fractional
  edge from making one side one pixel narrower than the other.

Explicit target resolution selects both `resolution-declaration` and `aspect-ratio-convert` in every
entry point. Explicit target aspect selects `aspect-ratio-convert`, including CLI and batch jobs.

Aspect protection only prevents new non-uniform scaling. It deliberately does not inspect or repair
artwork and geometry mistakes already present in the source skin.
- `blend-fallback`: replace LR2-only destination `blend=5` with beatoraja-compatible `blend=2`, and disable structurally matched duplicate option-label fades that darken in beatoraja.
- `lr2-dark-mode`: add a skin-selectable `OFF` / `DARK` / `DARK2` route without consuming
  destination option fields. Hidden groups retain their parsed objects and draw order while all
  destination frames are made transparent. `DARK` hides static `SRC_IMAGE` groups. `DARK2` keeps
  only IMAGE timers `2`, `3`, `48`, `49`, `50..69`, and `140`, hides secondary HUD groups, and
  preserves SUDDEN and LIFT lane covers as opaque black boards. A rhythm line remains visible when
  timer `140` is declared on either its source or destination. Lane-cover-related numbers `14`,
  `15`, `313`, and `314` remain visible. Notes, lines, judgment lines, gauges, BGA, `NOWJUDGE`, and
  `NOWCOMBO` are not filtered. Static beatoraja judgment-line compatibility images are recognized
  by matching both their source slice and destination geometry to the native judgment line, and
  remain visible in both DARK variants. This rule is opt-in.

Key-laser sprite reconstruction, gauge-label sprite ordering, and arbitrary custom LIFT images stay
manual because their source rectangles and animation ordering are skin-specific.
