# Balance sweep — night 1, Learned strategy

Reproducible measurement of the fuel/fare/upgrade economy using the playtest
bot on the seeded determinism seam. Every row replays exactly from its seed.

## Method

45 single-shift runs (15 seeds x 3 progression tiers), night 1, strategy
`learned`, zero action delay. Raw rows in `verification/balance_sweep.csv`.

```powershell
nightmare_shift.exe --bot --bot-shifts=1 --bot-delay-ms=0 `
    --bot-strategy=learned --seed=<1..15> `
    [--bot-almanac-level=0 | --bot-almanac-level=2 | --bot-almanac-level=3 --bot-all-skills]
```

## Results

| tier      | unlocks                    | survived | avg earnings / quota | avg rides | avg fuel left |
|-----------|----------------------------|---------:|---------------------:|----------:|--------------:|
| baseline  | nothing                    |     7/15 |            119 / 150 |       2.3 |           50% |
| studied   | almanac Lv.2               |     8/15 |            134 / 150 |       2.4 |           48% |
| mastered  | almanac Lv.3 + full skills |    13/15 |            209 / 150 |       2.6 |           61% |

Death reasons across all tiers: **15 of 17 deaths are passenger meltdown**
("The passenger's need became uncontrollable"); the other 2 are hidden-rule
violations. Zero guideline misreads — the Learned strategy consults the same
exception check the judge uses, so guideline deaths only appear under the
Coverage strategy.

## Findings (validate, don't tune yet)

1. **Meltdown is a near-monoculture failure mode.** 88% of deaths. Fuel and
   the clock are never the killer at this tier (avg 48-61% fuel left, ~2.4
   rides of a ~12-ride clock); shifts end because a passenger breaks. Any
   difficulty tuning should start at the need-growth vs relief seam, not at
   fuel/fare prices.
2. **Almanac Lv.2 buys almost no survival** (+1 win in 15) despite being the
   tier that reveals route preferences and exceptions. Its relief pathway —
   reading the passenger's own exception — is gated behind the exception
   liveness roll (authored 0.3-0.6 per ride), so some rides *cannot* be
   settled no matter how well they are read. That is authored design, but it
   compounds: the passenger whose exception rolled dormant is the passenger
   most likely to account for the meltdown column above. Worth a deliberate
   look at whether `exceptionRelief` should have a weaker fallback when the
   exception is dormant.
3. **The full tree nearly doubles the pass rate** (47% -> 87%) and pushes
   earnings 39% over quota — most of it fare skills and wards rather than
   knowledge. Progression pays, and end-tier night 1 is comfortable without
   being free.
4. **Baseline night 1 is a coin flip** (47%). Reasonable for a roguelite
   opening, but note the quota gate does some of the killing: several
   "survived" shifts still fail on earnings < 150.

## Caveats

- One night, one strategy, 15 seeds per tier: enough to rank tiers and spot
  the meltdown monoculture, not to resolve differences under ~15 points.
- The bot soothes at most once per leg and never refuels tactically midway
  through a losing night; a human may do better or worse.
- Later nights (rising quota, difficulty) are unmeasured; rerun with the
  same seeds after any tuning to diff the exact same nights.
