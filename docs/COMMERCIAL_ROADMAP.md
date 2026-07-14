# Nightmare Shift — Prototype → Commercial Roadmap

Date: 2026-07-14. Based on a full audit of the codebase (~14,000 lines of Rust, ~85 KB of JSON content).

**Overall verdict:** the *engine* is closer to commercial than the *game*. The service architecture (ride lifecycle, passenger need-state machine, guideline engine, weather, rules, items) is real and interlocking. What's prototype-scale is everything around it: content volume, run structure, audio (0%), settings/accessibility (0%), tests (0), and release packaging. The repo's own `standing.md` estimate of 5–8 months to commercial quality looks right, with audio/mood explicitly called out as the make-or-break.

Legend: 🔴 blocking for any commercial release · 🟠 expected by genre/store standards · 🟡 strongly recommended polish

---

## 1. Core content — the engine is starved 🔴

The signature mechanic (read tells → follow or break passenger guidelines) has only **2 guidelines** in `assets/guidelineData.json` against a 335-line engine (`src/engine/guideline_engine.rs`) that supports detection probability, trust, and even false tells for skilled players.

- [x] 🔴 Author **15–25 guidelines** with exceptions, tells, and consequences (currently 2). *(Done 2026-07-14: expanded to **18 guidelines** in `guidelineData.json`, each with authored tells/conditions/consequences/rewards. All 16 passengers now have at least one live guideline exception — critically, #2 (`shortcut_time_critical`) and #3 (`gps_passenger_warning`) had `stateProfile.exceptionId`s with no matching guideline, so their signature decision moments never fired before. Verified all 18 deserialize against the Rust `Guideline` types.)*
- [x] 🔴 Replace procedurally-assembled mid-ride events with an **authored event deck**. Today events are stitched from 4 hardcoded title/description templates in `ride_service.rs:662-830` with generic choices; `src/data/event.rs` is types-only and no event JSON exists. Target: dozens of hand-written, branching events in `assets/`. *(Done 2026-07-14: added `assets/eventData.json` with **16 authored events** (10 any-route + 6 route-specific for shortcut/scenic/police), each with hand-written horror flavor and 3 tradeoff choices. New `EventTemplate` type + `load_events()` loader; `generate_mid_ride_event` now draws a route-eligible template weighted by `weight`, still appends the passenger-ability choice when almanac+skill are unlocked, and shuffles. Stripped all ~15 shipping `[DEBUG] println!` lines from the event path and removed the dead `debug_route_options` helper and the now-unused `RouteCosts.risk_tags` field. Verified all 16 deserialize.)*
- [x] 🟠 **Make passenger selection data-driven** — spawn weights by weather/time/season were hardcoded per passenger id 1–16 in `passenger_service.rs:132-269`. *(Done 2026-07-14: added a `spawnWeighting` object to `passengerData.json` (weather/time/season multiplier maps + `heavyWeatherBoost`, `supernaturalTimeScaling`, `stormLatenightBoost`, `fogNightBoost` flags) and rewrote the four `PassengerService` modifier functions to read it. Migration is purely additive; a full-grid parity test (16 passengers × every weather/intensity/phase/activity/season) confirmed the data-driven weights are byte-identical to the old hardcoded logic. Adding a passenger is now pure data.)*
- [ ] 🟠 Grow the passenger roster beyond 16 (target 30–40 for a roguelite). *(Now unblocked by the data-driven selection above — new passengers need only a JSON entry. Gated on portrait art: ids 1–16 have `assets/passengers/N.png`; new entries fall back to the procedural silhouette until art exists.)*
- [x] 🟠 Make the 24 locations mechanically distinct — currently name/description/atmosphere + one risk number (`locationData.json`), pure flavor. *(Done 2026-07-14: added a `fareModifier` (0.85–1.6, hand-tuned per location by risk/remoteness) to all 24 locations and wired it into `GameEngine::calculate_fare` via the destination lookup in `complete_ride`. Destinations are now economically distinct — remote/dangerous drop-offs (Crossroads 1.5, End of the Line 1.6, Abandoned Hospital 1.45) pay a premium; safe civic ones (Police Station 0.85, Seminary/Office 0.9) pay less. Combined with the existing per-pickup `risk_level` (which already drives route risk), both endpoints of a ride now carry mechanical weight. All passenger pickups/destinations verified to resolve to real locations, so the modifier always applies. Migration is additive; default is 1.0 for any future location. **Further depth** (per-location fuel/distance, spawn-affinity tables, destination-risk on the drop-off leg) remains a natural follow-up.)*
- [x] 🟠 Expand shift rules beyond the current 20 (`shiftRulesData.json`) — depth per rule is good; count is thin for replay. *(Done 2026-07-14: **20 → 28 rules**. Added 8 fully-enforceable rules — 4 basic + 4 conditional — on the four cab-action keys that were already input-bound (Y/H/A/S → `use_wipers`/`drive_dark`/`use_ac`/`stop_vehicle`) and label/phrase-supported but had **no rules referencing them**, so they enforce via the existing `check_rule_violation` path with zero code changes. Grows the basic pool 5→9 and conditional 3→7 (both are what `generate_shift_rules` samples each shift), directly increasing shift-to-shift variety. Note: the action vocabulary is a fixed hardcoded set (input_service + game/rules.rs); further rule growth needs new cab actions there or condition support in the rule engine.)*
- [x] 🟡 Move item names out of hardcoded string arrays in `item_service.rs:102-155` into data. *(Done 2026-07-14: extracted the six per-category item-name arrays into `assets/itemPoolData.json` (24 names across ghost/vampire/demon/occult/holy/common). New `ItemPools` type + `load_item_pools()` loader; `select_item_for_passenger` now picks from the data pool via `ItemPools::pick`, and the six `random_*_item` functions are gone. Pools thread through `generate_drop`/`check_trade_offer` from `GameData`.)*
- [x] 🟡 Delete the dead legacy `assets/gameData.json` (empty stub). *(Done 2026-07-14: removed; confirmed zero source references — nothing `include_str!`s or loads it.)*

## 2. Game structure, progression & endings 🔴

Every run is the same single static 480-minute shift (`constants.json`), ending in a binary Success/GameOver screen (`game.rs:276-304`). Difficulty scaling exists but is mild (`experience / 10`, capped at 4, only adds 1–2 conditional rules).

- [ ] 🔴 Design a **run structure / campaign**: escalating nights (night 1/2/3…), night modifiers, a run arc, possibly a "final night." This is the largest single design task.
- [ ] 🔴 **Fix or cut the dead half of the skill tree.** Only the 6 `ability_unlock` skills do anything (checked at `ride_service.rs:745`); all `stat_boost`/`passive_bonus` skills (Hybrid Injection, Expanded Tank, Reinforced Chassis, Glimpse, Basic Warding, Silver Tongue, Negotiator) are **purchasable no-ops** — no code reads `fuel_consumption`, `max_fuel`, `tip_multiplier`, `shop_discount`, `reveal_hidden_chance`, `sanity_resistance`, or `hazard_damage`.
- [ ] 🔴 Implement or cut the **shop** and **sanity** systems that skill descriptions reference — neither exists in `game_state.rs`.
- [ ] 🟠 Multiple endings / epilogues and a framing story for the driver. Narrative today is episodic only (16 passenger backstories, unlocked by RNG); no through-line, no character arcs, one ending, no credits.
- [ ] 🟠 Deepen the skill tree past 12 shallow nodes toward meaningful build identity/branches.
- [ ] 🟠 **Seeded runs**: the toolkit ships a serializable `SeededRng` built exactly for this, but the game uses only macroquad's global unseeded RNG (zero hits for `srand`/`SeededRng` in `src/`). Adopting it enables daily runs, challenge seeds, reproducible bugs, and deterministic tests (§7).
- [ ] 🟡 Run modifiers / relic-style variety and unlock-gated content for long-tail replay.
- [ ] 🟡 Resolve the guideline-timeout no-op: the 30 s decision timer expires without forcing a decision (`game.rs:507-517`, "We can't modify state here").

## 3. Audio — completely absent 🔴

The single largest gap for a horror title. `assets/sounds/` contains only a README; macroquad's audio feature isn't even enabled in `Cargo.toml`; there is zero playback code. Notably, the data layer already defines `audioCue` strings per passenger/guideline (`passengerData.json`, `guidelineData.json`) that are deserialized (`data/passenger.rs:100`) but never played.

- [ ] 🔴 Add an audio backend (macroquad audio or kira-class crate) with a mixing/volume layer.
- [ ] 🔴 Ambient bed: engine hum, rain/weather layers, city night ambience.
- [ ] 🔴 Dynamic music/tension score that escalates with passenger instability and route risk (the state signals already exist and drive the visual vignette).
- [ ] 🔴 Horror stingers wired to the existing `audioCue` data; meltdown/violation/game-over sounds.
- [ ] 🟠 UI SFX (clicks, confirms, page turns) and per-passenger voice-adjacent sound design (breathing, whispering — several cues are already named, e.g. `distressed_breathing`).
- [ ] 🟠 Subtitles/captions for audio cues (accessibility, §5).

## 4. Art & visual identity 🟠

Rendering is competent but almost entirely procedural vector primitives plus 16 static passenger portraits. `assets/image_prompts.json` specifies **27 planned assets — only the 16 portraits exist**; menu background, logo, HUD icons (fuel/money/time), item icons, and 3 driving backgrounds (city/forest/industrial) are all faked with primitives.

- [ ] 🟠 Produce (or formally cut) the remaining planned art: logo, main-menu hero art, driving backgrounds, HUD/item icons.
- [ ] 🟠 Portrait treatment pass: mapping is by bare index `1..16.png` with no verified character mapping; consider light animation (blink/sway/reaction states) — static portraits undercut the horror reads.
- [ ] 🔴 **Fix the inert screen shake**: `get_offset()` is computed then discarded into `_shake_x, _shake_y` at `src/game/render.rs:17` — `shake(15.0, 0.5)` on game-over produces no visible motion.
- [ ] 🟠 Event-tied visual feedback (flashes on rule violation, meltdown framing, jump-scare moments) — dread currently rests on text plus faint overlays and reads "tense management sim" more than horror.
- [ ] 🟡 Lighting dynamics (headlight cones, darkness levels per route) — currently static shapes; animate the static background rain streaks.
- [ ] 🟡 Resolve the emoji problem: `en.json` is full of emoji the bitmap font can't render, so `ascii_trimmed()` strips them at draw time (`ui/components.rs:9-12`) — intended HUD iconography silently disappears. Replace with drawn icons.

## 5. Settings, accessibility, onboarding & input 🔴

There is **no settings/options screen at all** — no grep hit for settings/fullscreen/rebind/colorblind/accessibility anywhere in `src/`. The pause menu offers only Resume / Return to Menu.

- [ ] 🔴 Options screen: audio volumes (once audio exists), fullscreen/resolution in-engine (currently only the HTML wrapper has a fullscreen button), brightness/gamma.
- [ ] 🔴 **In-engine tutorial/onboarding + help screen.** The game drops straight into the shift; controls are documented only in `index.html`, invisible to the native build.
- [ ] 🟠 Key rebinding (bindings hardcoded in `input_service.rs`, with semantic overlaps on A/S/W).
- [ ] 🟠 Accessibility: reduced-motion toggle (glitch/vignette/pulse effects), colorblind-safe palette check, text scaling, high-contrast mode, captions for audio cues.
- [ ] 🟠 Gamepad support (zero controller code today) — expected for a Steam release.
- [ ] 🟡 Touch support for the web build: several actions are letter-key-only (E/M/T/W/Y/H/A/S) with no on-screen equivalents, so parts of the game are unplayable on touchscreens.

## 6. Localization 🟡

The i18n layer is structurally solid (typed loader, `en.json` v1.1.0, UI chrome routed through it) but functionally single-language, and most actual text bypasses it.

- [ ] 🟠 Externalize narrative content: passenger names/dialogue/backstories, rules, route/event text live as raw English in data JSON and Rust; game-over strings are hardcoded (`game.rs:292,296`), as are strings in `game_screens.rs:611` and `item_service.rs:317`.
- [ ] 🟡 Add a language selector and at least one non-English locale to prove the pipeline; pick a font strategy that covers target scripts.

## 7. Testing, determinism & code health 🔴

**Zero tests exist** — no `#[test]` anywhere, no `tests/` dir — so CI's `cargo test` step is green and meaningless. The README's own test wishlist (rule selection, tell detection, route choice, fuel, rewards, failure conditions) is unimplemented. The 558-line playtest bot (`src/bot.rs`) is a genuinely capable smoke harness with CI-friendly exit codes but is not run in CI.

- [ ] 🔴 Introduce the seedable RNG seam (§2), then build the deterministic test suite the README describes.
- [ ] 🔴 Wire the playtest bot into CI as a runtime smoke gate (`--bot` run per push; exit code 2 = stuck already exists).
- [ ] 🟠 Balance/simulation harness: use the bot's almanac-sweep to validate economy (fuel/fare/upgrade curves) at scale before tuning.
- [ ] 🟠 Decompose the files violating the repo's 800-line hard limit: `game_screens.rs` (1,922), `meta_screens.rs` (1,133), `menu_screens.rs` (1,052), `ride_service.rs` (991), `weather_service.rs` (807).
- [ ] 🟠 Finish the README's meta-vs-shift state separation — overlay flags (`show_rules`, `show_pause_menu`, …) still live on the `Game` god-struct next to simulation state (`game.rs:15-28`).
- [ ] 🟠 Strip ~20 shipping `[DEBUG] println!` calls from `ride_service.rs` (749-823, 837-908, 935-990).
- [ ] 🟡 Replace the two `expect()` panics (`loader.rs:69,109`) and the `unwrap()` at `ride_service.rs:695` with graceful fallbacks; surface data-load errors in-game instead of stderr-only.

## 8. Persistence & save robustness 🔴

Saves are a single autosave (`SaveData v1`: meta stats, skills, almanac, leaderboard, achievements) to app-data JSON (native) or localStorage (WASM).

- [ ] 🔴 **Corrupt-save handling**: any load failure silently wipes all progression (`game.rs:44` falls back to fresh stats). Add backup/quarantine of the bad file + user notification.
- [ ] 🟠 Save migration framework — a version field exists and newer saves are rejected, but there's no upgrade path for schema changes.
- [ ] 🟠 Mid-run save/resume (a short-session game still needs interrupt-safety; requires the seeded RNG to be serializable — the toolkit RNG already is).
- [ ] 🟠 Cloud-save hook for Steam; save export/import for WASM (localStorage is wiped by cache clears).

## 9. Platform, packaging & release engineering 🟠

Ships as raw zips: Windows exe (22 MB + 20 MB assets) and WebGL (80 MB zipped — very heavy for web-first).

- [ ] 🔴 **Asset size**: 16 portraits at ~1.1–1.4 MB each (~19 MB) are embedded via `include_bytes!` (`ui/core.rs:410-425`) **and** duplicated in `assets.zip`. Compress/resize and ship once — the biggest single download-size win, especially for WASM.
- [ ] 🔴 Add `[profile.release]` to `Cargo.toml` (lto, opt-level, strip) — currently absent entirely, unlike sibling projects.
- [ ] 🟠 Windows exe polish: icon + version/company metadata via `build.rs` (`winresource`); installer (NSIS/MSI or Steam depot) and code signing.
- [ ] 🟠 Steam/itch integration: `steamworks` bridge for the existing internal achievements (`player_stats.rs:264`), cloud saves, rich presence; itch butler pipeline for DRM-free.
- [ ] 🟠 Crash reporting: panic hook → log file + opt-in upload; the only feedback channel today is the web-page bug widget (`index.html:191`) — native players have nothing.
- [ ] 🟡 Opt-in gameplay telemetry to tune difficulty/economy.
- [ ] 🟡 Release automation in CI (artifact upload, tagged releases) — CI builds but publishes nothing.
- [ ] 🟡 Web shell: real download-progress UI (spinner only today), responsive canvas (fixed 1920×1080), mobile detection.

## 10. Commercial & go-to-market 🟠

- [ ] 🟠 Store presence: page copy, capsule art, trailer, press kit (14 UI screenshots + `catalog_thumbnail.png` exist; nothing else).
- [ ] 🟠 External playtesting program with structured feedback (the bot proves stability, not fun); difficulty/economy tuning from real sessions.
- [ ] 🟠 Pricing/positioning decision: short-session horror roguelite vs. premium narrative; affects how much §1/§2 content is "enough."
- [ ] 🟡 Demo build (Steam Next Fest-style: first night only), wishlisting funnel, launch-window plan.
- [ ] 🟡 Legal basics: EULA/privacy (needed if telemetry/crash upload ships), licensing audit for fonts/audio you add.

---

## Suggested sequencing

1. **Foundations first (cheap, unblocks everything):** seeded RNG seam → deterministic tests + bot in CI; strip debug prints; fix screen shake; corrupt-save handling; `[profile.release]` + portrait compression.
2. **The two zero-percent pillars:** audio system + settings/accessibility/tutorial. Horror doesn't work silent, and stores expect options.
3. **Content sprint:** guidelines 2→20+, authored event deck, data-driven passenger spawning, wire-or-cut the dead skills (with shop/sanity decision).
4. **Structure:** multi-night campaign, endings, narrative framing.
5. **Release engineering:** Steam/itch integration, packaging, crash reporting, store assets, external playtests.
