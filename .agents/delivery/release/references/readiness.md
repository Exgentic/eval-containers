# Release-readiness go/no-go checklist

The gate a release manager walks before cutting a tag. Derived from the
2026-04-18 readiness verdict; generalized into the reusable criteria.

The single artifact that certifies a commit is the fleet report,
`cargo test --test fleet -- --ignored` →
`.agents/verification/fleet/report.md`. This checklist is what a human
reads alongside that report to decide ship / hold. It does not replace
the report; it explains how to read its verdict.

## Verdict classification

Matches `.agents/RULES.md:14` (the fleet report's red/yellow/green
verdict) and `tests/fleet/RULES.md:4`:

- **GREEN** — every mechanical gate green AND every procedural audit
  green. Ship.
- **YELLOW** — some yellow findings, no reds. Ship-ready *with
  documented gaps*: every yellow MUST be enumerated with a root cause
  and a reason it is not release-blocking (typically: local-only infra
  flake, upstream-tool false positive, or a deferred-to-CI re-run).
- **RED** — any red finding. NOT ship-ready. MUST be fixed before
  tagging. No release MAY ship with a red verdict
  (`.agents/RULES.md:14`).

## Gate matrix — every gate green or justified-yellow

Walk every row; a release is go only when no row is red. The VERIFY
step numbers are the procedural steps each gate corresponds to.

| VERIFY step | Gate | Executor |
|---|---|---|
| 4 | `cargo fmt --check` + `cargo clippy -- -D warnings` | mechanical |
| 5 | `cargo test` (rule-engine + CLI unit tests) | mechanical |
| 6 | Structural validation (dir count ↔ label) | mechanical (sanity) |
| 7 | Compose parse — every `docker compose config` parses | mechanical (sanity) |
| 8 | Dockerfile rule catalog — 0 red | mechanical (sanity) |
| 9 | Trajectory rule catalog — fixtures healthy | mechanical (sanity) |
| 10 | Count reconciliation — README ↔ filesystem | mechanical (sanity) |
| 11–14 | Build sweep — every buildable image | mechanical (build) |
| 15 | Replay — recorded-trajectory sweep | mechanical (replay) |
| 16–17 | Live smoke — full stack end-to-end | release (live) |
| 18–20 | Upstream probes — every pinned ref resolves | release (upstream) |
| 21 | hadolint | external lint |
| 22 | gitleaks | external lint |
| 23 | Dockerfile audit (new files) | procedural audit |
| 24 | Trajectory audit (new fixtures) | procedural audit |
| 25 | Fleet audit | procedural audit |
| 30–31 | README presence — every benchmark + agent | mechanical (sanity) |
| 35 | Fleet report rendered | mechanical (fleet) |

Gate authorities by category:
`tests/sanity/RULES.md` (steps 4–10, 30–31),
`tests/build/RULES.md` (11–14),
`tests/replay/RULES.md` (15),
`tests/live/RULES.md` (16–17),
`tests/upstream/RULES.md` (18–20),
`tests/fleet/RULES.md` (35).

## Outstanding-findings policy

A YELLOW ships only if every outstanding finding is documented. For
each finding record:

1. **Root cause** — why it is yellow not green.
2. **Why not red** — the reason it does not block (local infra flake,
   upstream-tool false positive, deferred re-run on CI).
3. **Single home to fix** — where the fix lands when it is addressed.

Recurring legitimate yellows from past cycles (examples, not a
standing waiver — re-justify each release):

- Build/live sweeps that fail under **local concurrent-network
  contention** (podman-on-macOS) but pass on CI under real Docker on
  Linux. Per `.agents/delivery/RULES.md`'s principle "CI builds the fleet, humans
  build one thing at a time", the authoritative full sweep is the CI
  run; document the local caveat in
  `.agents/verification/build/known-broken.md`
  (`tests/build/RULES.md:6`).
- **hadolint heredoc false-positives** on `RUN python3 <<'PYEOF'`
  blocks — an upstream parser limitation, not a Dockerfile defect.
  Separate the real low-severity warnings from the false positives.
- **Mutable `:latest` tag** in first-party `COPY --from=` before the
  registry is live — tighten to a digest once published.
- **Cost tracking reading 0** on certain LiteLLM API paths — noted in
  the live known-broken manifest; does not fail a run on its own
  (`tests/live/RULES.md`, trace-inspection rule 27).

## Recommendation form

State the recommendation explicitly: **ship**, or **hold for CI to
validate the full build + live sweep**. Justify against the ladder:

- No red signals anywhere in the contribution-verification ladder
  (steps 4–15).
- Any local build/replay failures are network-bound, not structural —
  confirmed by targeted samples building cleanly.
- The full fleet matrix + fresh live fixtures are validated (or
  explicitly deferred to the next CI run, with the reason).
