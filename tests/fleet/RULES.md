# Fleet test rules

The fleet category is the **aggregator**. It doesn't run any new tests
— it probes the state of every other category and renders a single
report with a green/yellow/red verdict. It's what a release manager
reads before cutting a tag.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Pure aggregation.** The fleet test MUST NOT build images, run
   containers, or make network calls. It reads log files, walks the
   filesystem, and calls the other categories' mechanical rule
   catalogs. Anything slower than reading a file is forbidden.

2. **Probe-don't-run.** If a category (build, replay, upstream, live)
   produces a persistent log or report file, the fleet test PROBES
   that file and interprets it. It does NOT re-run the category. For
   a fresh probe, the operator runs that category out of band first,
   then re-runs fleet.

## Output

3. **One file: `tests/fleet/report.md`.** The report has three sections:
   mechanical gates (one row per category), procedural audits
   (embedded from `audit-*.md` if present), and a final verdict.

4. **Verdict classification** (matches top-level `/RULES.md` rule 13):
   - **Green** — every mechanical gate green AND every procedural
     audit green.
   - **Yellow** — some yellow findings, no reds. Ship-ready with
     documented gaps.
   - **Red** — any red finding. Not ship-ready. Must be fixed before
     release.

## Gate probing

5. **Each category owns its probe.** The fleet test calls one probe
   function per category (`probe_sanity`, `probe_build_sweep`,
   `probe_replay`, `probe_upstream`, `probe_live`). Each probe returns
   a `Gate { step, name, phase, verdict, detail, duration_ms }`.

6. **Known-broken aware.** Build sweep probe compares failures to
   `tests/build/known-broken.md` and returns yellow if every failure
   is within the manifest. Live probe does the same against
   `tests/live/known-broken.md`. Replay probe treats fixtures listed
   in `replay/fixtures/broken.json` as informational.

## Release manager contract

7. **Regeneration is mechanical.** `cargo test --test fleet --
   --ignored` MUST regenerate `report.md` from scratch. The operator
   MUST NOT hand-edit the report between runs.

8. **Commits under a tag preserve the report.** When cutting a release,
   the operator MUST commit the final `report.md` alongside the tag so
   the release artifact carries its own verification record.
