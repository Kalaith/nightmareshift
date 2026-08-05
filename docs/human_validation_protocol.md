# Human release-gate protocol

Use the published browser build at the commit under test. Do not explain a
route, passenger tell, exception, cab action, or outcome before the player has
made the relevant decision. Record uncertainty in the player’s own words.

## Blind first-session cohort

The five new players already required by `first_three_runs_playtest.md` form
this cohort. For the first attempt, only tell them that the in-game tutorial
and handbook are available. Observe whether they can:

1. start a standard shift;
2. read the briefing and accept a passenger;
3. state the major tradeoff among the four route cards;
4. inspect the rules and use one cab action;
5. make a guideline decision; and
6. explain the itemized post-ride result.

Do not convert a prompted success into an unassisted success. Put task failures
and prompts in `critical_ui_missing` or `notes` in
`docs/verification/session_playtests.csv`.

## Experienced full-campaign cohort

Use players who understand the rules and progression, ideally after the blind
cohort has finished its three-run sessions. Ask each player to attempt a full
six-night campaign without coaching. Record one row per campaign attempt and
capture:

- night reached and whether the campaign completed;
- whether fuel changed a route/refuel plan;
- whether Comfort’s prevented harm was visible;
- whether route identities remained situational;
- any quota-clock collapse on later nights; and
- whether each informative audio cue had a usable visual/caption equivalent.

Run at least three experienced full-campaign sessions. This is a validation
sample, not a balance statistic; verbatim observations matter more than a win
rate.

## Ratings and evidence handling

Use `yes`, `partly`, or `no` for comprehension ratings. The report generator
accepts only those values, checks that the five first-time players have three
distinct attempts each, and keeps the release gate pending until the required
rows exist. Do not copy bot outcomes into either human CSV.

Generate the current gate report with:

```powershell
.\scripts\summarize-human-playtests.ps1
```

The generated report is `docs/verification/human_playtest_report.md`.
