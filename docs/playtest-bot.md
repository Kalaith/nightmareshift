# Playtest Bot

Nightmare Shift includes a small state-aware bot for smoke-testing the main gameplay loop. It runs inside the game process, so it is not dependent on canvas image recognition or brittle browser coordinates.

## Run

```powershell
.\scripts\run-playtest-bot.ps1 -Shifts 3 -Strategy coverage
```

Equivalent cargo command:

```powershell
cargo run -- --bot --bot-shifts=3 --bot-strategy=coverage --bot-delay-ms=150
```

Seed almanac knowledge for bot runs:

```powershell
.\scripts\run-playtest-bot.ps1 -Shifts 1 -Strategy learned -AlmanacLevel 2
```

Sweep from no almanac knowledge through full almanac knowledge:

```powershell
.\scripts\run-bot-almanac-sweep.ps1 -Strategy learned -RunsPerLevel 1 -FreshStats
```

## Strategies

- `coverage`: cycles routes, event choices, and guideline decisions to exercise more branches.
- `conservative`: prefers normal and police routes, follows guidelines, and tries to survive.
- `learned`: uses almanac route knowledge when available, otherwise falls back to coverage.

## Unlock Sweep

`run-bot-almanac-sweep.ps1` backs up the native save, runs one or more bot shifts at almanac levels `0` through `3`, stops at the first successful shift, and restores the original save afterward. Use `-FreshStats` to start each level without the player's current save data.

## Output

The bot logs each action and a shift summary to stderr with a `[BOT]` prefix. It exits with code `0` after the requested number of shifts. It exits with code `2` if the game appears stuck in the same phase for more than 12 seconds.
