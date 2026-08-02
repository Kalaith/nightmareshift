# TODO — Nightmare Shift

## Audio

- Enable a macroquad/kira-class audio backend with a mixing and volume layer. There is no playback code at all, and `assets/sounds/` holds only a placeholder README.
- Ambient bed: engine hum, weather layers, city-night atmosphere.
- Dynamic tension score escalating with passenger instability and route risk — the state signals already drive the visual vignette.
- Wire the existing `audioCue` strings in `passengerData.json` / `guidelineData.json` to real stingers; add meltdown, violation and game-over sounds.
- UI sound effects and per-passenger sound design (breathing, whispering — several cues are already named).

## Settings, accessibility & onboarding

- Add an options screen: audio volumes, in-engine fullscreen/resolution, brightness. Nothing of the kind exists; the pause menu offers only Resume and Return to Menu.
- Add an in-engine tutorial and help screen — controls are documented only on the web page, invisible to the native build.
- Key rebinding; bindings are hardcoded in `input_service.rs` with semantic overlaps on A/S/W.
- Accessibility pass: reduced-motion toggle for the glitch/vignette/pulse effects, colorblind-safe palette check, text scaling, high contrast, captions for audio cues.
- Gamepad support.
- Touch equivalents for the letter-key-only cab actions (E/M/T/W/Y/H/A/S), which make parts of the web build unplayable on a touchscreen.

## Determinism & testing

- Build the daily-challenge/seed-entry UI on the finished determinism seam. The developer half ships: `--seed N` / `NIGHTMARE_SHIFT_SEED` re-arm the stream at every run start (briefing shows "Seeded run: N"), and two same-seed bot runs produce byte-identical action logs. Known caveat for *human-paced* replay: hazard regeneration (60 real seconds) and weather turnover (real-minute durations) draw on wall-clock schedules, so very long shifts can interleave extra draws — moving those to sim-time is part of the mid-run save work below.
- Wire the playtest bot into CI as a runtime smoke gate — it already exits 2 when stuck. (Blocked on infrastructure: `rust-ci.yml` is centrally managed in `rust_management` and runs headless, so the macroquad window needs an xvfb/software-GL story before this can land.)
- Act on the balance sweep findings in `docs/balance_sweep.md` — the harness is built and reproducible (45 seeded shifts across three tiers). Headline: 88% of deaths are passenger meltdown, almanac Lv.2 barely buys survival because exception liveness can roll relief unreachable, and the full tree takes night 1 from 47% to 87%. Tuning decisions are yours; rerun the sweep with the same seeds to diff any change.

## Persistence

- Add a save-migration path — a version field exists and newer saves are rejected, but there is no upgrade route for schema changes. (Corrupt and newer-version saves are now quarantined — renamed aside with a menu notice — instead of being silently overwritten; migration would let the quarantined file be brought forward.)
- Mid-run save and resume (depends on the serializable seeded RNG above).
- Save export/import for the web build, since localStorage dies with a cache clear; cloud-save hook if Steam happens.

## Content

- Grow the passenger roster past 16 toward 30–40. Selection is data-driven now, so new fares are a JSON entry — gated on portrait art, since ids 1–16 have `assets/passengers/N.png` and anything new falls back to the procedural silhouette.
- Per-night run modifiers ("Blood Moon: fares +20%, +1 rule") selected in `begin_night` and applied through the existing `minimum_earnings` / `SkillModifiers` / rule-count hooks.
- Longer authored epilogues, branching endings, and a credits sequence — the campaign's endings are currently mechanically distinct but thin.
- Further location depth: per-location fuel and distance, spawn-affinity tables, destination risk on the drop-off leg.

## Art

- Produce or formally cut the remaining planned assets in `image_prompts.json`: logo, menu hero art, three driving backgrounds, HUD and item icons. Only the 16 portraits of 27 planned assets exist.
- Portrait treatment pass — the mapping is by bare index with no verified character correspondence; light animation (blink, sway, reaction states) would sharpen the horror reads.
- Event-tied visual feedback on rule violations and meltdowns; the game currently reads more "tense management sim" than horror.
- Lighting dynamics: headlight cones, per-route darkness, animated rain streaks.
- Replace the emoji in `en.json` with drawn icons — the bitmap font cannot render them, so `ascii_trimmed()` strips the intended HUD iconography at draw time.

## Localization

- Externalize narrative content: passenger names, dialogue and backstories, rules, and route/event text are raw English in data JSON and Rust, and the game-over strings are hardcoded.
- Route the ~145 hardcoded UI strings through `en.json` — the entire pause menu, dossier module, briefing panel headers, driving-screen risk labels, and skill-tree/leaderboard/almanac labels all bypass the localization struct (~90 keys) today.
- Add a language selector and one non-English locale to prove the pipeline, with a font strategy covering the target scripts.

## Packaging & release

- Compress and resize the 16 portraits (~19 MB) and ship them once — they are embedded via `include_bytes!` *and* duplicated in `assets.zip`, which is the single biggest download-size win.
- Windows executable polish: icon and version metadata via `build.rs`, plus an installer and code signing.
- Crash-report upload (opt-in), gated on the privacy-policy decision below. The local half is done: `macroquad_toolkit::crash::install_crash_log` writes every native panic to `crash_log.txt` beside the save file. (A per-project `[profile.release]` was also once listed here; dropped — profiles come from the workspace root, a member's is ignored with a warning, and no sibling carries one.)
- Steam/itch integration for the existing internal achievements, cloud saves, and a DRM-free pipeline; release automation in CI, which builds but publishes nothing.
- Web shell: real download-progress UI, responsive canvas (fixed at 1920×1080 today), mobile detection.

## Commercial

- Store presence: page copy, capsule art, trailer, press kit.
- External playtesting with structured feedback — the bot proves stability, not fun.
- Decide pricing and positioning (short-session horror roguelite vs. premium narrative); it determines how much content is "enough".
- Demo build covering the first night, wishlist funnel, launch-window plan.
- Legal basics: EULA and privacy policy if telemetry or crash upload ships, plus a licensing audit for any fonts and audio added.
