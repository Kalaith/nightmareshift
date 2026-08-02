# Balance sweep — night 1, Learned strategy (second measurement)

Reproducible measurement of the survival/relief economy using the playtest
bot on the seeded determinism seam. Every row replays exactly from its seed.
This supersedes the first sweep, taken before the meltdown-seam work; that
table is kept below for the diff.

## Method

60 single-shift runs (15 seeds x 4 progression tiers), night 1, strategy
`learned`, zero action delay. Raw rows in `verification/balance_sweep.csv`.

```powershell
nightmare_shift.exe --bot --bot-shifts=1 --bot-delay-ms=0 `
    --bot-strategy=learned --seed=<1..15> --bot-fresh-stats `
    [ --bot-skills=stereo_1,stereo_2,climate_1,climate_2,upholstery_1,upholstery_2
    | --bot-almanac-level=2
    | --bot-almanac-level=3 --bot-all-skills ]
```

**`--bot-fresh-stats` is new and load-bearing.** The bot used to play
against the real save, which meant two corruptions at once: lifetime shift
count drives `suggested_difficulty`, so a well-used save raised the measured
difficulty; and a baseline tier inherited whatever skills and almanac the
save had accumulated — a "baseline" run on a mastered save silently measured
the mastered tier. The first sweep predates the flag and its absolute
numbers should be treated accordingly; its finding of a meltdown monoculture
stands, since meltdown dominated regardless of the confound. The bot's
unstudied tactics were also made humane in the same change set (blind
desperation gambles only at Critical, at most one per ride, and
comfort-equipped cabs actually press their safe soothing controls).

## Results

| tier      | unlocks                       | survived | avg earnings / 150 | avg rides | avg fuel left | deaths |
|-----------|-------------------------------|---------:|-------------------:|----------:|--------------:|--------|
| baseline  | nothing                       |     2/15 |                 74 |       2.9 |           42% | 7 violation-gamble, 4 meltdown, 1 misread, 1 clock |
| comfort   | comfort branch only (~$4,350) |     3/15 |                 94 |       4.1 |           38% | 5 clock/quota, 4 violation, 2 misread, 1 hidden |
| studied   | almanac Lv.2                  |    12/15 |                196 |       4.7 |           24% | 1 hidden, 1 clock, 1 meltdown |
| mastered  | almanac Lv.3 + full skills    |    13/15 |                197 |       3.4 |           34% | 2 hidden rule |

First sweep, for the diff (45 runs, three tiers, unisolated save):

| tier      | survived | avg earnings | deaths |
|-----------|---------:|-------------:|--------|
| baseline  |     7/15 |          119 | meltdown-dominated |
| studied   |     8/15 |          134 | meltdown-dominated |
| mastered  |    13/15 |          209 | (15 of 17 deaths across tiers were meltdown) |

## Findings

1. **The meltdown monoculture is broken.** 5 of 60 runs (8%) end in
   meltdown, against 88% of deaths before. Three changes did it, each
   visible in the columns: the six severed rule→guideline links restored
   (every soul but the Collector has a cab-action relief route), the
   once-per-shift brink grace (meltdown telegraphs one leg before killing),
   and the dormant-exception fallback (a break made on a real tell whose
   exception rolled dormant is spared the 0.7 death and pays half relief —
   the "weaker fallback" the first sweep asked after). Death causes now
   spread across violations, hidden rules, the clock, and misreads.
2. **The comfort branch does exactly its one job.** Comfort-only runs have
   *zero* meltdown deaths and ride 40% longer than baseline, but survival
   barely moves (2→3) because the failures relocate to the quota clock.
   It is a relief valve, not a win button — priced right at ~$4,350 for the
   six nodes, roughly two successful early nights.
3. **Knowledge is the survival axis.** Studied (Lv.2 almanac, no skills)
   jumps to 12/15 on its own: the verdict line and exception matching are
   what turn night 1 from a gauntlet into a shift. The full tree adds
   earnings and converts the last meltdown into ward absorptions; at
   mastered, only hidden rules kill (2/15).
4. **Baseline night 1 is now harsh — deliberately.** 2/15 with the run's
   bank/lore payout intact on death matches the design's arc: first runs
   end early and *fund* the loop (a dead baseline night still banks ~$37
   and its lore, and every survived ride inscribes almanac progress). If
   external playtesting reads it as unfair rather than roguelike, the
   levers are the brink count and the violation death probabilities, not
   the fuel economy — fuel still never kills anyone.
5. **The unstudied bot models a gambler, not every player.** All 7 baseline
   "violation" deaths are single desperation presses of a forbidden control
   at Critical — rational against imminent meltdown, but a cautious player
   who refuses the gamble dies to the meltdown instead. Either way the
   run ends; the almanac is what actually changes the odds.

## Caveats

- One night, one strategy, 15 seeds per tier: enough to rank tiers and
  causes of death, not to resolve differences under ~3 wins.
- Later nights (rising quota and difficulty) are unmeasured; The Last Fare
  (night 6) is measured only anecdotally — seeds 2, 5, 7, 8 deliver Death
  with 82–94% fuel left, which is why TODO calls it gentle.
- Rerun with the same seeds and `--bot-fresh-stats` after any tuning to
  diff the exact same nights.
