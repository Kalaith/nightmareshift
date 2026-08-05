# Nightmare Shift Review Resolution Plan

Date: 2026-08-05

## Purpose

This plan turns the August 2026 design, balance, feature-interaction, and UI
review into an implementation sequence. It is deliberately evidence-led: the
game's core systems are connected and well tested, but balance is uneven and
the current presentation obscures much of the useful information those systems
produce.

The intended outcome is a game in which:

- a new driver can understand why a run was lost and what can improve the next
  attempt;
- each route and progression branch has a distinct, worthwhile role;
- later nights test several interacting resources instead of collapsing almost
  entirely into quota and clock pressure;
- supernatural consequences are legible, atmospheric, and accessible; and
- the UI looks cohesive beside the passenger portrait art and remains usable at
  common desktop browser sizes.

## Review Baseline

### Validation completed

- All 239 gameplay unit tests passed.
- The source-size standards test passed.
- Tests already cover rule generation and conflicts, hidden-rule reachability,
  passenger tells and need, guideline exceptions, skills, items, trades,
  protection, reputation, weather, hazards, route quotes, almanac reveals,
  persistence, outcomes, and seeded night generation.
- The existing night-one balance sweep reports 2/15 baseline survival, 3/15
  with the comfort branch, 12/15 at almanac level 2, and 13/15 with full
  knowledge and skills.
- A fresh 30-shift campaign sample with full knowledge and skills produced
  26/30 successful shifts and three completed six-night campaigns.
- A fresh 30-shift campaign sample with level-three almanac knowledge and no
  skills produced 22/30 successful shifts and one completed campaign.

The two campaign samples are directional evidence, not release-grade balance
statistics. They were not a fixed-seed matrix and do not replace human
playtesting.

### Confirmed strengths

The central loop is coherent. Passenger need and tells feed guideline choices;
cab actions and comfort upgrades provide relief; almanac knowledge explains
preferences, tells, rules, and relief; passenger traits combine with purchased
abilities during events; and weather, hazards, reputation, route mastery,
items, and night modifiers all reach real gameplay calculations.

Route costs and fare estimates are calculated through the same paths used to
charge and reward the player. The game does not have an obvious disconnected
marquee feature. The work below should preserve that system integrity rather
than broadly rewriting it.

### Issues to resolve

1. **Knowledge has a progression cliff.** Night-one survival rises from 2/15
   at baseline to 12/15 at almanac level 2. This may support the intended
   roguelite arc, but without in-engine onboarding it can read as arbitrary
   punishment instead of earned mastery.
2. **Later pressure concentrates on time and quota.** Almost every failure in
   the two campaign samples was an out-of-time or missed-quota result. Fuel,
   meltdown, and ordinary rule violations became secondary once the passenger
   was understood.
3. **Fuel has less strategic weight than advertised.** Existing measurements
   explicitly report that fuel does not kill runs, while four survival-tree
   nodes and the refuelling economy are devoted to it.
4. **Normal Route lacks a clear niche.** Its base cost is 7 fuel, 25 minutes,
   risk 1, and fare multiplier 1.0. Police Route costs 10 fuel but is faster at
   23 minutes, safer at risk 0, and richer at fare multiplier 1.1. When fuel is
   weak, Police Route is the stronger choice on nearly every axis.
5. **Fare variance can decide a night.** Passenger base fares range from $12 to
   $100 before route, reputation, destination, modifier, and skill multipliers.
   Rare high-value combinations can make a quota trivial while ordinary fare
   mixes remain clock-bound.
6. **Comfort benefits are mechanically real but poorly expressed.** The
   comfort-only sweep removed meltdown deaths but barely improved survival
   because failures moved to the quota clock. Players need visible proof of
   what the cab prevented.
7. **The UI understates the game.** Low contrast, small text, repeated outlined
   panels, duplicated status information, provisional vector scenery, missing
   icon glyphs, and limited consequence feedback make a modern systems-driven
   game look and feel older than it is.
8. **Onboarding and accessibility are incomplete.** There is no in-engine
   tutorial, help or options screen, cab-action touch controls, text scaling,
   high-contrast mode, reduced-motion setting, or audio-caption path.
9. **The horror presentation is incomplete.** Authored audio cues have no
   playback system, and rule violations, exceptions, wards, and meltdowns do
   not yet receive distinctive audiovisual reactions.

## Product Principles

- Do not make early runs easier until player comprehension has been measured.
  Improve explanation and feedback first, then tune probabilities if deaths
  still feel arbitrary.
- Keep the four routes situational. No route should be a default answer after
  the player understands the system.
- Prefer pressures that create a decision over pressures that merely shorten a
  bar. Fuel should change available plans before it becomes a sudden loss.
- Preserve knowledge as the supernatural survival axis, but smooth its value
  across Observed, Studied, and Mastered tiers.
- Show the source of every important modifier. The player should be able to
  distinguish a known fact, an almanac inference, and an unknown risk.
- Build shared UI components before restyling individual screens.
- Preserve deterministic seams and add metrics before changing balance values.

## Phase 1: Campaign Balance Instrumentation

### Work

1. Extend the playtest output and measurement scripts to record:
   - run and seed;
   - night reached and campaign completion;
   - progression tier;
   - active night modifier;
   - passenger and fare contribution;
   - route selection frequency;
   - fuel, time, quota progress, and wards at the end of each shift; and
   - precise failure cause.
2. Add a fixed-seed campaign matrix covering at least:
   - baseline;
   - comfort branch only;
   - almanac level 1;
   - almanac level 2;
   - almanac level 3 without skills; and
   - almanac level 3 with all skills.
3. Report results by night as well as in aggregate. A high night-one success
   rate must not hide a campaign wall at night four, and failed campaigns must
   not continually overweight night one.
4. Record the distribution, not just the average, for earnings and time left.
   Include median, 10th/90th percentiles, and the share of earnings supplied by
   the single highest-paying fare.
5. Add a small human-playtest form for the first three runs. Capture whether
   each death was predicted, understood afterward, and perceived as avoidable.

### Exit criteria

- The same seed and tier reproduce the same authored night and comparable bot
  decisions.
- Campaign completion and night-reach rates are visible for every tier.
- Route usage and fare outliers can be identified without reading raw bot logs.
- At least five new players complete three attempts each before final early-run
  tuning.

## Phase 2: Focused Balance Corrections

Make these changes one at a time and rerun the Phase 1 matrix after each group.

### Route identities

- Give Normal Route a defensible identity. First test a larger fuel advantage
  and lower passenger-need growth or encounter volatility; do not simply make
  Police Route worse on every axis.
- Keep Shortcut as the urgent, fuel-efficient, high-risk option.
- Keep Scenic as the slower, higher-fare preference play.
- Keep Police as the low-supernatural-risk option with a meaningful fuel,
  passenger, checkpoint, or rule cost.
- Verify that no route exceeds roughly half of learned-strategy selections
  across the campaign matrix unless a modifier or passenger preference
  deliberately creates that night-specific bias.

### Progression curve

- Measure human use of the level-one need and threshold information before
  changing almanac costs.
- If the knowledge jump remains too abrupt, move one actionable tell or a
  partial preference clue into Observed rather than weakening Studied.
- Review the 1/3/5 lore costs against actual lore earned per failed and
  successful run. The target is an early useful purchase without immediate
  roster-wide mastery.
- Surface ability-carrier requirements and already-studied carrier counts in
  the skill UI so an ability purchase cannot look disconnected.

### Resource pressure

- Make low fuel alter route availability, refuelling opportunity cost, or
  emergency options before considering harsher fuel consumption.
- Give survival upgrades measurable campaign value without making an unlucky
  fuel roll an instant death.
- Review quota growth and fare variance together. Apply caps or diminishing
  stacking only if the percentile report shows that a single fare routinely
  trivializes a night.
- Preserve the comfort branch as protection, but show avoided need growth,
  brink saves, and ward-like interventions in the result summary.

### Exit criteria

- Every route owns at least one common situation in which it is the best
  informed choice.
- No single route dominates the fixed-seed learned-strategy matrix.
- Baseline deaths remain threatening but are understood and considered
  avoidable by most first-time testers.
- Progression improves campaign reach in visible steps rather than one large
  almanac-level cliff.
- Fuel changes decisions in a meaningful share of shifts without becoming a
  leading opaque death cause.

## Phase 3: Shared UI Foundation

This phase precedes screen-by-screen polish so the refresh does not become a
collection of incompatible local drawing helpers.

### Work

1. Define a shared design system in `src/ui/core.rs` and
   `src/ui/components.rs`:
   - type scale and minimum readable sizes;
   - spacing and panel rhythm;
   - semantic colors for safe, warning, danger, occult, money, and unknown;
   - button, card, badge, meter, tooltip, and modal states;
   - keyboard focus, mouse hover, disabled, selected, and urgent treatments;
   - responsive anchors and safe margins; and
   - reduced-motion variants.
2. Replace stripped emoji with a small drawn icon atlas for fuel, time, fare,
   risk, weather, rules, inventory, wards, lore, and cab controls.
3. Establish responsive layout tiers rather than assuming a fixed 1920x1080
   canvas. The first supported targets are 1920x1080, 1600x900, 1366x768, and a
   narrower desktop-browser viewport.
4. Add a UI capture scene for every new shared component and its important
   states.
5. Check text and semantic colors against high-contrast and common color-vision
   deficiencies. Never rely on color alone for route risk or rule danger.

### Visual direction

Use a restrained late-night dispatch aesthetic: dark dashboard surfaces,
warm meter and streetlight accents, cold weather information, and controlled
supernatural color intrusions. Reduce decorative borders and use contrast,
spacing, depth, and typography to establish hierarchy.

The polished passenger portraits should remain the visual focus. Surrounding
cab, road, and city art must be brought to a compatible level rather than
flattening the portraits to match the procedural scenery.

### Exit criteria

- All gameplay screens use the same component vocabulary.
- Primary text and interactive states remain readable at every target size.
- Keyboard focus is always visible.
- No intended icon passes through `ascii_trimmed()` or appears as a missing
  glyph.
- Reduced motion and high contrast can be applied without per-screen forks.

## Phase 4: In-Shift UI Redesign

### Status and decision hierarchy

- Replace the duplicated HUD and central stat panel with one compact dashboard
  showing fuel, time, quota progress, rides, weather, wards, and passenger
  need.
- Make current phase and required action the strongest element on screen.
- Keep Rules and Inventory available without competing with the current
  decision.
- Add contextual mouse and touch controls for every cab action while preserving
  keyboard shortcuts.

### Ride requests and routes

- Present pickup, destination, fare range, known traits, reputation, and
  knowledge level in a single passenger dossier hierarchy.
- Redesign route options as four consistent decision cards with chips for
  quoted time, fuel, fare, risk, preference, hazard, and rule interaction.
- Label information as Known, Inferred, or Unknown.
- Explain disabled routes and quote floors directly on the card.
- Make passenger preferences readable without requiring color memory.

### Passenger state and consequences

- Give need escalation a persistent, named meter when knowledge permits and a
  deliberately ambiguous treatment when it does not.
- Show exactly how much a comfort action relieved and whether it is exhausted
  for the ride.
- Give guideline follow, justified exception, violation, hidden-rule reveal,
  ward absorption, reputation change, and brink grace distinct reactions.
- Summarize the last leg's fuel, time, fare, stress, items, and rule effects at
  drop-off without forcing the player to reconstruct them from log text.

### Exit criteria

- A tester can explain the major tradeoff between all four routes before
  choosing.
- A tester can identify why passenger need changed and what action affected it.
- No critical information is duplicated in competing panels.
- Every cab action is usable with keyboard, mouse, and touch.

## Phase 5: Meta Screens, Onboarding, and Accessibility

### Main menu and briefing

- Introduce cohesive hero art and a stronger run-state panel.
- Separate Standard Shift, Daily Shift, and Seeded Run clearly while retaining
  fast keyboard start.
- Present the night's quota, difficulty, modifier, weather, hazards, and
  starting resources as a concise forecast rather than a wall of equal-weight
  cards.

### Skill tree

- Replace the three long columns with category tabs or a true node map and a
  persistent selected-skill detail panel.
- Show prerequisites, magnitude, carrier requirements, current bank balance,
  and expected gameplay effect before purchase.
- Keep the lore-to-bank exchange visible without giving it equal emphasis to
  progression.

### Almanac

- Replace the two-column stack of expanded dossiers with a portrait grid or
  compact roster and one selected-passenger dossier.
- Make the next level's revelations and cost explicit.
- Distinguish knowledge bought with lore from discoveries earned during play.

### Onboarding and options

- Add a short first-shift tutorial that teaches route quotes, passenger tells,
  guideline decisions, cab actions, and the difference between following a
  rule and recognizing an exception.
- Add an always-available help and controls screen.
- Add options for text scale, high contrast, reduced motion, brightness,
  fullscreen/resolution, and—when audio lands—master, ambience, music, and
  effects volume.
- Prepare captions and visual equivalents for authored audio cues.

### Exit criteria

- A new player can start, finish a ride, inspect rules, use a cab action, and
  understand the post-ride summary without consulting the web page.
- Skill and almanac purchases state both their cost and their practical effect.
- Every screen is navigable without a mouse.
- Accessibility settings persist and apply consistently.

## Phase 6: Horror and Audio Presentation

### Work

- Add an audio backend and mixer before producing the full sound library.
- Implement ambience layers for engine, weather, and city night.
- Drive tension music from passenger need, route risk, and rule pressure.
- Connect existing authored `audioCue` values to stingers and captions.
- Add portrait reaction states, mirror or glass disturbances, headlight and
  rain treatment, violation flashes, ward effects, and escalating cabin
  darkness.
- Replace the provisional taxi and roadside presentation with cohesive driving
  backgrounds and dashboard framing.
- Respect reduced-motion, brightness, and caption settings throughout.

### Exit criteria

- A rule violation, justified exception, ward absorption, brink state, and
  meltdown are distinguishable without reading the event log.
- Audio adds information but is never the sole carrier of required information.
- Passenger portraits and environment art read as one deliberate visual world.

## Phase 7: Final Validation

1. Run all unit and source-standards tests.
2. Run the complete fixed-seed campaign matrix and compare it with the Phase 1
   baseline.
3. Conduct blind first-session tests and experienced full-campaign tests.
4. Capture every verification scene at each target viewport and inspect text,
   overlap, clipping, focus, and contrast.
5. Validate keyboard, mouse, touch, and any added gamepad paths.
6. Run `publish.ps1` with no parameters from the project root.
7. Verify the published browser build and refresh `catalog_thumbnail.png` from
   the finished title screen.

## Release Gates

The review is considered resolved when all of the following are true:

- No route is strategically redundant or dominant across the campaign matrix.
- Campaign difficulty rises by night without becoming exclusively a quota-clock
  test.
- Knowledge progression improves survival in understandable steps.
- Fuel and comfort upgrades change decisions and visibly explain their value.
- New-player deaths are usually understood and perceived as avoidable.
- All major systems expose their effects through the UI.
- The main menu, gameplay, routes, skill tree, almanac, and outcome screens use
  the refreshed design system.
- Required actions are accessible by keyboard, mouse, and touch, with text
  scaling, high contrast, and reduced motion available.
- Horror consequences have distinctive visual feedback and captioned audio
  equivalents.
- Tests, the fixed-seed balance matrix, UI capture review, and `publish.ps1`
  all pass.

## Deliberate Non-Goals

- Do not expand the passenger roster until the existing 16 passengers are
  clear, balanced, and presented cohesively.
- Do not broadly rewrite deterministic gameplay systems that are already
  connected and covered by tests.
- Do not add new dependencies solely for visual polish when macroquad or the
  shared `macroquad-toolkit` can reasonably own the capability.
- Do not tune solely to bot win rates. The bot validates reachability and
  relative pressure; human testing decides whether the game is understandable,
  tense, and fun.
