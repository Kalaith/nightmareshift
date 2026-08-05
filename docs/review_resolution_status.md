# Review resolution status

Date: 2026-08-05

## Engineering and content work

| Phase | Status | Evidence |
| --- | --- | --- |
| 1. Campaign instrumentation | Complete | Fixed 15-seed, six-tier campaign matrix; distributions, per-night failures, route/fare/resource telemetry; first-three-runs form |
| 2. Balance corrections | Complete | Normal route identity, quota curve, learned strategy, Comfort category; no learned route dominates beyond the intended roughly-half bound |
| 3. Shared UI | Complete | Scalable type/wrapping, semantic palette, drawn icon set, focus/hover/disabled states, four responsive tiers, contrast/reduced-motion settings |
| 4. In-shift UI | Complete | Compact status bar, known/inferred/unknown route language, disabled reasons and quote floors, persistent need state, mouse/touch cab controls, itemized leg receipts |
| 5. Meta/onboarding/accessibility | Complete | Generated hero art, forecast briefing, tabbed skill dossier, compact Almanac roster/dossier, first-shift handbook, persistent options and audio controls |
| 6. Horror/audio | Complete | Mixer, ambience/tension layers, original WAV set, authored cues, captions, distinct reaction frames, painted cockpit environment |
| 7. Automated validation | Complete | 246 tests plus source gate, 90-campaign matrix, 140-capture viewport matrix including dedicated shared-component scenes, Windows/WebGL no-parameter publish, clean real-browser startup/input transition |

## Balance result

The fixed-seed report in `docs/verification/campaign_matrix_report.md` records
visible progression steps from baseline through Almanac levels and all skills.
Normal route supplies measurable need relief, Comfort supplies measurable
prevention, refuelling has an explicit opportunity cost, and the all-skills
learned strategy's largest route share is 51.5% (Scenic), within the plan's
"roughly half" allowance rather than a universal default.

## Human-only release gates

These gates remain pending external participants and are not represented as
completed by bot runs or screenshots:

- five new players completing three attempts each, with predicted/understood/
  avoidable-death responses in `docs/first_three_runs_playtest.md`; and
- blind first-session plus experienced full-campaign playtests.

No further balance tuning should be claimed from those gates until the forms
contain real participant observations. The implementation, capture harness,
and measurement artifacts needed to conduct them are complete. Use
`docs/human_validation_protocol.md` and run
`scripts/summarize-human-playtests.ps1` to validate and summarize collected
rows without replacing qualitative review.
