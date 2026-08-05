# Review resolution requirement audit

Date: 2026-08-05

This audit maps every work item and acceptance gate in
`docs/review_resolution_plan.md` to current authoritative evidence. “Pending
human evidence” is not a euphemism for a bot result: those claims need external
participants before they can pass.

## Phase 1 — campaign instrumentation

| Requirement | Result | Evidence |
| --- | --- | --- |
| Record seed/run, night reach, tier, modifier, passenger/fare, route use, end resources, wards and exact cause | Pass | `src/bot/report.rs`, `src/bot/campaign.rs`, `scripts/run-campaign-matrix.ps1` |
| Fixed matrix for baseline, Comfort, Almanac 1/2/3, and all skills | Pass | `docs/verification/campaign_matrix_report.md` covers 15 seeds × 6 tiers = 90 campaigns |
| Per-night and aggregate results | Pass | Campaign report night tables and aggregate tier tables |
| Earnings/time distributions, P10/P50/P90, largest-fare share | Pass | Distribution columns in the campaign report |
| First-three-runs human form | Pass | `docs/first_three_runs_playtest.md` |
| Reproducible authored night and comparable bot choices | Pass | Fixed seeds plus deterministic campaign tests and report |
| Five new players × three attempts | **Pending human evidence** | Empty participant form; no synthetic observations accepted |

## Phase 2 — focused balance corrections

| Requirement | Result | Evidence |
| --- | --- | --- |
| Normal identity; Shortcut urgent/risky; Scenic slower/richer; Police safer/costlier | Pass | Route definitions, quote cards, Normal need relief, and matrix route breakdown |
| No learned route above roughly half without specific bias | Pass | All-skills largest share is Scenic at 51.5%, reported with modifier/passenger context |
| Measure level-one human use before changing costs | Correctly deferred | Almanac costs were not changed; human-use measurement remains part of the external form |
| Conditional partial Observed clue only if human evidence shows a cliff | Not triggered | Knowledge tiers already produce visible campaign steps; human evidence may still request tuning |
| Review 1/3/5 lore costs and expose ability carriers/counts | Pass | Progression report plus selected-skill dossier and carrier-count tests |
| Low fuel changes routes and refuelling opportunity cost | Pass | Affordability, stranding, discount, and refuel-cost tests; blocked-route capture |
| Survival upgrades have measurable value without opaque fuel deaths | Pass | Tier telemetry and explicit prevented-harm receipt |
| Review quota and fare variance; cap only if percentiles justify it | Pass | Percentile/largest-fare report; no unjustified blanket cap added |
| Comfort visibly reports relief, brink saves, and wards | Pass | `RideImpact`, drop-off receipt, component capture |
| First-time deaths understood/avoidable by most testers | **Pending human evidence** | Requires the five-player form |

## Phase 3 — shared UI foundation

| Requirement | Result | Evidence |
| --- | --- | --- |
| Shared type, spacing, semantic colors, responsive margins, motion variants | Pass | `src/ui/core.rs`, `src/ui/primitives.rs` |
| Button/card/badge/meter/tooltip/modal vocabulary and states | Pass | Shared core/components plus deterministic state gallery |
| Drawn icons for fuel, time, fare, risk, weather, rules, inventory, wards, lore and cab | Pass | `UiIcon` atlas and `ui_core` capture |
| Four target layout tiers | Pass | 1920×1080, 1600×900, 1366×768, 900×720 matrix |
| Dedicated capture scene for every shared component and important state | Pass | `ui_core`, `ui_status`, `ui_passenger`, `ui_completion` at all four viewports |
| High contrast, color-safe non-color labels, visible keyboard focus | Pass | Accessible capture, labelled semantics, focus-state capture and shared button code |
| No intended icon is stripped or becomes a missing glyph | Pass | Drawn atlas plus loader/source tests |

## Phase 4 — in-shift UI

| Requirement | Result | Evidence |
| --- | --- | --- |
| One compact fuel/time/quota/rides/weather/wards/need hierarchy | Pass | Gameplay, driving, warded captures and status component |
| Current phase/action dominates; Rules/Inventory remain secondary | Pass | Complete viewport matrix |
| Keyboard, mouse and touch cab actions | Pass | Input service plus common button hit regions in Rules; scroll drag-release protection |
| Passenger dossier and four consistent route cards | Pass | Ride-request and three route-state captures |
| Known/Inferred/Unknown, disabled reasons, quote floors, text preference labels | Pass | Route/dossier implementation and captures |
| Persistent named/ambiguous need, exact comfort relief and exhaustion | Pass | Dossier state tests, driving UI and receipt telemetry |
| Distinct rule/exception/ward/reputation/brink reactions | Pass | Reaction scenes, captions, state-owned reaction kind |
| Itemized fuel/time/fare/stress/item/rule drop-off effects | Pass | `CompletionSummary` and dedicated capture |
| Tester can explain route tradeoffs and need changes | **Pending human evidence** | Tutorial is implemented; comprehension claim requires observation |

## Phase 5 — meta screens, onboarding, accessibility

| Requirement | Result | Evidence |
| --- | --- | --- |
| Hero art, clear standard/daily/seeded starts, concise briefing forecast | Pass | Main-menu, seed-entry, briefing captures |
| Tabbed skill tree with persistent detail, prerequisites, magnitude, carriers, bank and effect | Pass | Skill-tree implementation, capture and carrier tests |
| Compact Almanac roster, selected dossier, next revelations/cost, earned-vs-bought provenance | Pass | Almanac implementation, persistence tests and capture |
| First-shift tutorial covers quotes, tells, decisions, cab actions, exceptions | Pass | `draw_help_options` tutorial path |
| Always-available help/controls | Pass | Menu and paused paths return correctly |
| Text scale, contrast, motion, brightness, fullscreen, captions and four mixer controls | Pass | Persisted accessibility settings, options capture, settings tests |
| Resolution handling | Pass | Browser canvas responds to viewport; fullscreen is player controlled; four desktop sizes validated |
| Every screen keyboard navigable | Pass | Sequential shared focus plus screen/phase shortcuts and advertised-key tests |
| New player completes and understands a ride without the web page | **Pending human evidence** | Blind first-session observation required |

## Phase 6 — horror and audio

| Requirement | Result | Evidence |
| --- | --- | --- |
| Audio backend/mixer, engine/weather/city ambience, need/risk/rule tension | Pass | `src/audio.rs`, authored WAV library and audio tests |
| Authored audioCue stingers plus captions/visual equivalents | Pass | Cue loading, event queue, caption rendering and tests |
| Reaction portraits, glass/headlight/rain treatment, violation/ward/darkness effects | Pass | Four reaction captures and cockpit renderer |
| Cohesive driving background/dashboard art | Pass | Generated cockpit asset and all in-shift captures |
| Reduced motion, brightness and captions respected | Pass | Shared presentation state and accessible capture |
| Consequences distinguishable without the log; audio not sole carrier | Pass | Distinct frame/color/caption combinations in reaction scenes |

## Phase 7 and release gates

| Requirement | Result | Evidence |
| --- | --- | --- |
| Unit and source tests | Pass | 246 unit tests + source gate |
| Complete fixed-seed matrix | Pass | 90-campaign report |
| Blind first-session and experienced full-campaign tests | **Pending human evidence** | External participants required |
| Every scene at every viewport inspected | Pass | 140 raw captures and four labelled contact sheets |
| Keyboard/mouse/touch validation; no added gamepad path | Pass | Shared input/hit regions, input tests and matrix report |
| `publish.ps1` with no parameters | Pass | Windows and WebGL publish recorded in status |
| Published browser verification and catalog thumbnail | Pass | Deployment verifier, clean browser run, refreshed thumbnail |

Automated and implementation release gates pass. The unresolved gates are all
claims about a human player’s prediction, comprehension, avoidability, or
full-campaign experience. They cannot be proven from source, screenshots, or a
bot campaign and therefore remain open.
