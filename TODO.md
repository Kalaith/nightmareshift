# TODO — Nightmare Shift

## Unconnected systems (wiring audit, 2026-08-02)

Code and data that exist but never reach the player. Verified against the source; file references are to the anchor site, not every occurrence.

### Simulation outputs that go nowhere

- Time-of-day is frozen at Dusk. `weather_service/calendar.rs:52` hardcodes start hour 18 and derives elapsed time from real wall-clock hours, so `Night`/`LateNight` never occur in play — permanently disabling weather rule 104, the `PoliceCheckpoint` hazard, the night route-risk bonus and Scenic-latenight penalty, the low-ambient-light headlight fuel penalty, and night-weighted passenger spawns. The `DEFAULT_START_HOUR = 20` seeded in `shift.rs` is overwritten on the first frame (and lives in `ui/core.rs` despite being a sim constant).
- Season is a constant Fall (`DEFAULT_MONTH = 10`), so the spring/summer/winter passenger spawn weights, the winter hazard-chance bonus, and the winter conditions branch never fire; the `Temperature` enum and `Season.description` are computed but displayed nowhere.
- The false-tell system is doubly dead: its gate requires `rides_completed > 20` but a 480-minute/100-fuel shift caps out around 12–13 rides, and even if it opened, the merge dedupe rejects the cloned tell it tries to insert (`guideline_engine.rs:86-103`, `343-365`).
- Shift-rule consequences half-ignore their data: `ConsequenceType::Death`/`Survival` are no-op arms in the rule path (`game/rules.rs:204-205`), so 11 of the 13 rules whose follow reward is `survival` pay nothing for being kept, and the authored death `probability` is ignored (the shift ends unconditionally two lines later). `Item` consequences hardcode a "Crumpled Note", discarding the authored value and description. (The guideline path at `rules.rs:543-551` handles both properly.)
- ~150 authored `Consequence.description` strings in `shiftRulesData.json`/`guidelineData.json` ("You resisted the passenger's pull…") never reach a screen — the shown message is built from the guideline title instead.
- Passenger reputation is applied (fare multiplier, risk modifier) but never shown on any screen and never persisted — it dies at run end. `negative_choices` and `last_encounter` are write-only outside tests.
- `GameState.pending_route_dialogue` is declared, initialised and reset but never written or read (`game_state.rs:460`). `PlayerStats.total_play_time` is accumulated and persisted but no screen shows it.
- The status bar under-reports wards: `wards_in_hand()` counts only rule-immunity/supernatural-protection charges, not carried items with `protectiveProperties` that `ProtectionService` actually spends — a driver holding a Blessed Medallion reads "0 wards".
- Never-constructed variants: `ProtectionType::SafePassage`/`LuckyEncounters` (no item may author them — a test enforces it) and `EventConsequence::None` (matched, never produced).

### Input/UI wiring gaps

- SPACE on the mid-ride event screen skips the event without applying any `EventConsequence` and enters a phase the engine comments "should not happen"; SPACE on drop-off silently declines a pending trade; `AcceptTrade`/`DeclineTrade` are the only actions dispatched with no screen/phase guard.
- ESC cannot pause during `RideRequest` (it declines the ride instead, with no on-screen pause button), and the briefing screen has no path back to the menu.
- `PassengerCard`'s Accept/Decline controls are dead — the only call site passes `show_controls = false`; the ride-request screen builds its own buttons.
- The loading screen advances after 2 frames, making it the never-legible sole reader of `localization.meta` (language/code/version).

### Dead data (authored JSON → nothing)

- `constants.json` `SCREENS` and `STORAGE_KEYS` are JS-port leftovers with no serde mapping; the `nightshift_*` storage keys don't even match the hardcoded `nightmare_shift*` names in `persistence.rs`. Also `RISK.EXTREME_RISK` and `CONSECUTIVE_ROUTE.VIOLATION_THRESHOLD` are read only by tests.
- `shiftRulesData.json` `defaultSafety` (13 rules) and the `temporary: true` flag on the Mayor's Decree rule modification are silently dropped at load — no struct field, and unlike the other known-dropped keys, undocumented.
- `Guideline.visible` and `Guideline.difficulty` are deserialized but never read (five authored difficulty tiers decide nothing); `exceptions[].probability` is never rolled — `check_exception_conditions` is fully deterministic; `ExceptionCondition.description` is unread.
- `Location.description` (all 24 locations), `AlmanacLevel.description`, and `EventTemplate.id` are loaded and never consumed.
- `skillTreeData.json`: `effect.type` distinguishes `stat_boost`/`mechanic_unlock`/`passive_bonus` but dispatch is entirely on `effect.target`; `effect.value` is discarded for the five passive ability unlocks; `third_eye_1`'s text promises a per-ride reveal but the roll happens once per shift.
- `tells[].type`: only `verbal` is ever branched on; the 20 behavioral/visual/environmental tells are treated identically to each other.

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

- Thread a state-owned `SeededRng` through the ~35 gameplay call sites while leaving visual effects on macroquad's global generator. Seeded/daily runs, mid-run save/resume, and full-shift scenario replays all depend on this seam; `srand`-only was rejected as a misleading half-measure since `effects.rs` draws from the global stream every frame.
- Wire the playtest bot into CI as a runtime smoke gate — it already exits 2 when stuck.
- Use the bot's almanac sweep as a balance harness to validate the fuel/fare/upgrade economy before tuning.

## Persistence

- Handle corrupt saves: any load failure currently wipes all progression silently. Quarantine the bad file and tell the player.
- Add a save-migration path — a version field exists and newer saves are rejected, but there is no upgrade route for schema changes.
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

## Code health

- Decompose the five files over the 800-line hard limit: `engine/item_service.rs` (1242), `state/game_state.rs` (1062), `engine/game_engine.rs` (940), `screens/game_screens/dossier.rs` (888), `engine/passenger_state_machine.rs` (810).
- Separate meta-screen state from shift state — `show_rules`, `show_inventory` and `show_pause_menu` still sit on the `Game` struct beside simulation state, so pause, upgrades and results can leak into route simulation.
- Replace the two `expect()` panics in `loader.rs` and the `unwrap()` in `ride_service` with graceful fallbacks, and surface data-load errors in-game rather than only on stderr.

## Packaging & release

- Compress and resize the 16 portraits (~19 MB) and ship them once — they are embedded via `include_bytes!` *and* duplicated in `assets.zip`, which is the single biggest download-size win.
- Add `[profile.release]` to `Cargo.toml` (lto, opt-level, strip); it is absent entirely, unlike sibling projects.
- Windows executable polish: icon and version metadata via `build.rs`, plus an installer and code signing.
- Crash reporting — a panic hook writing a log with opt-in upload. Native players currently have no feedback channel at all.
- Steam/itch integration for the existing internal achievements, cloud saves, and a DRM-free pipeline; release automation in CI, which builds but publishes nothing.
- Web shell: real download-progress UI, responsive canvas (fixed at 1920×1080 today), mobile detection.

## Commercial

- Store presence: page copy, capsule art, trailer, press kit.
- External playtesting with structured feedback — the bot proves stability, not fun.
- Decide pricing and positioning (short-session horror roguelite vs. premium narrative); it determines how much content is "enough".
- Demo build covering the first night, wishlist funnel, launch-window plan.
- Legal basics: EULA and privacy policy if telemetry or crash upload ships, plus a licensing audit for any fonts and audio added.
