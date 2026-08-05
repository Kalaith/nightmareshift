# Review viewport matrix

Date: 2026-08-05

The deterministic capture harness rendered 35 scenes at each supported target:

| Viewport | Captures | Result |
| --- | ---: | --- |
| 1920x1080 | 35 | Pass |
| 1600x900 | 35 | Pass |
| 1366x768 | 35 | Pass |
| 900x720 | 35 | Pass after responsive corrections |

The raw 140 PNGs are reproducible with `scripts/capture-review-matrix.ps1`
and deliberately ignored by Git. The same command now rebuilds four committed,
labelled contact sheets through `scripts/build-review-contact-sheets.ps1`,
preserving the inspected matrix without turning raw build evidence into
permanent repository weight.

## Scene coverage

Dedicated core-vocabulary, status-bar, passenger-card, and completion-summary
scenes; main menu, seed entry, standard and hazard briefings, waiting and refuelling,
ride request, normal/broke/blocked route choices, guideline and event choices,
rules with cab-action controls, inventory, trade offer and result, warded ride,
four supernatural reactions, pause, skill tree, almanac, leaderboard, standard
and accessible options, armed save deletion, game over, night completion, and
campaign completion.

## Inspection findings

- Initial 900x720 route cards allowed preference labels, risk tags, and quote
  floors to collide. The narrow route band now reserves independent columns
  and truncates descriptive risk tags to measured width.
- Initial 900x720 briefing art covered the fourth rule. Narrow briefings now
  reserve the vertical space for all four rules and suppress the decorative
  taxi when its scene strip is too short.
- Initial 125% handbook text overlapped the Back control. Dense help copy now
  uses tighter section rhythm while retaining the scaled glyph size.
- Skill tabs, the selected-skill dossier, compact Almanac roster, passenger
  dossier, route-disabled reasons, cab-action buttons, captions, focus borders,
  and outcome paragraphs remain within their panels at all four sizes.
- High contrast, 125% text, and reduced motion were captured together in the
  accessible-options scene.
- The component gallery separately captures semantic type and colors, all ten
  required drawn icons, default/hover/focus/disabled/selected/urgent button
  states, labelled meters and badges, tooltips, the status bar, the portrait
  card, and the itemized completion summary at every viewport.

Keyboard activation and visible focus are centralized in the shared button
component. Mouse and touch use the same hit regions; the toolkit scroll area
explicitly absorbs drag releases so a touch pan cannot purchase or activate an
underlying control. No gamepad path was added by this resolution.

## Published-browser verification

The preview deployment was loaded in a real browser after the final publish.
The loading overlay dismissed, the canvas rendered the generated title art at
1200x675, and keyboard Space opened the first-shift tutorial. A clean reload
reported zero console errors both before and after that input transition.

The browser pass exposed and drove fixes for two issues native capture could
not see: startup no longer asks an inactive document to exit fullscreen, and
the complete UI glyph set is rasterized before the first queued draw so
Macroquad does not retire its font-atlas texture midway through a WebGL batch.
The deployment verifier also accepts cache-busted runtime script URLs and
confirms `sapp_jsutils.js -> storage.js -> wasm` load order.
