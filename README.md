# Nightmare Shift

Nightmare Shift is a horror taxi survival game about driving supernatural passengers through a city where every fare may have dangerous rules.

You work the graveyard shift, observe strange behavior, choose routes, manage fuel and money, and try to survive until dawn.

## Gameplay

- Accept or decline passengers.
- Watch for tells that reveal hidden threats.
- Read and follow passenger-specific guidelines.
- Choose routes with different risks.
- Manage fuel, earnings, upgrades, and survival pressure.

## Goal

Complete the night without breaking the wrong rule, running dry, or letting a dangerous passenger catch you unprepared.

## Controls

The whole in-shift loop is playable from the keyboard.

- **Space** - start a shift, find a passenger, accept a fare, continue.
- **Esc** - decline a fare, pause mid-shift, or leave a meta screen.
- **1-4** - choose a route while driving, or a choice during a mid-ride event.
- **F / B** - follow or break a guideline during the timed decision.
- **R** - rules panel. **I** - inventory.
- **Cab controls** - E eye contact, M music, T accept tip, W window, Y wipers,
  H headlights, A air conditioning, S stop the cab.
- **Mouse** - everything above, plus using items, trading, and buying skills
  and almanac levels, which have no keyboard shortcut.

## Current Scope

Playable horror-shift loop with passenger tells, route choices, rules, fuel
pressure, earnings, and upgrades, over a five-night run whose quota and
difficulty rise each night. Knowledge is earned as well as bought — surviving
rides and noticing tells fill the almanac — and a fully mastered roster opens
the endgame: a sixth night with no quota and a single fare, Death himself,
whose delivery is the run's true ending.

Open work — audio, settings, the seeded-RNG seam, packaging and everything
between here and a commercial release — is tracked in `TODO.md`.

## Verifying A Deploy

`scripts/verify-deployment.ps1` checks a published build over HTTP: that every
file serves, that the wasm arrives as `application/wasm`, that every storage
import the binary asks for is registered by the shared `storage.js`, and that
the scripts load in the order the bridge needs.

The third of those is the one worth having. `mq_js_bundle` stubs a missing
import rather than failing, so a game whose save bridge does not match writes
into nothing and looks perfectly healthy doing it - a bug this catalogue has
shipped before.

It cannot tell you the game runs. A WebGL context, macroquad's start-up and
the first frame need a real browser, and nothing in the script executes
JavaScript. Open `http://127.0.0.1/games/nightmare_shift/` for that.

## Notes For Whoever Publishes This

`publish.ps1` packs `assets/` into an `assets.zip` beside the wasm, per this
project's `asset_packs.json`. Nothing fetches it: the JSON data is
`include_str!` and the sixteen passenger portraits are `include_bytes!`, so
the binary already carries everything and the generated `index.html`
references only the `.wasm`. That is about 19 MB of a 78 MB deploy.

Deleting `asset_packs.json` does not help - the publisher then copies the
same files loose, slightly larger. Skipping the assets entirely would need
an option the shared publisher does not have, so it is left as it is and
noted here rather than worked around.

