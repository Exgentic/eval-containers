---
name: verify
description: >-
  Walk the complete release verification of eval-containers end to end —
  preflight, sanity gates, build/replay/live sweeps, upstream + security scans,
  the three procedural audits, docs and CI checks, the fleet report, and the
  tag/publish/announce flow. Use this when cutting a release tag or asked to
  "run release verification" / "is this release-ready". This is the whole-pipeline
  driver; for the individual judgment-level passes it invokes, use the narrower
  audit skills next door (audit-dockerfile, audit-trajectory, audit-fleet,
  audit-rules-drift).
---

# Verify eval-containers for release

Eval-containers is verified on **two axes**:

- **Mechanical** — deterministic checks, run by `cargo test` or an external
  tool, producing pass/fail.
- **Procedural** — judgment-level review, walked by a human or a sub-agent
  following a checklist, producing a written verdict.

Both axes exist at three scales: per-file, per-image, whole-fleet. Mechanical
catches what is broken; procedural catches what is wrong but passes the rules.
Neither replaces the other. This skill is the **complete release walk**: every
step, every executor, every artifact. Nothing ships without walking it.

This skill drives the whole pipeline. Two of its phases delegate to narrower
skills: the procedural audits in steps 23–25 are the `audit-dockerfile`,
`audit-trajectory`, and `audit-fleet` skills, and the RULES drift check in step
29 is the `audit-rules-drift` skill.

## Rules this skill serves

- `doctrine/verification/RULES.md:1` — contribution verification: offline, no
  API keys, under 2 hours, reproducible on a clean clone (steps 4–15 are the
  contribution gate).
- `doctrine/verification/RULES.md:2` — release verification: every contribution
  gate **plus** the live fleet sweep, the procedural audits, and the upstream
  reachability check (this whole walk).
- `doctrine/verification/RULES.md:13` — mechanical > procedural > aspirational;
  this walk runs mechanical gates first and only spends judgment where rules
  cannot reach.
- `tests/sanity/RULES.md:1-7` — the fast offline gates in
  steps 4–10.
- `tests/build/RULES.md:3-6` — core-first build order, per-task
  skips, and known-broken diffing in steps 11–14.
- `tests/replay/RULES.md:7,10` — broken-fixture handling and
  core-image dependency in step 15.
- `tests/live/RULES.md:1-12` — the live smoke and sweep in
  steps 16–17.
- `tests/upstream/RULES.md:4-10` — pinned-reference probes and
  the 404-is-red / auth-is-yellow policy in steps 18–20.
- `tests/fleet/RULES.md:3-8` — regenerate the report
  mechanically, classify the verdict, commit it under the tag (steps 35–37, 43).

## Procedure

Walk the steps in order. Record each step's artifact or pass mark in
`tests/fleet/report.md` (the generated release artifact). A release is **not
ready** until every step below has been executed and recorded.

### Phase 1 — Preflight (steps 1–3)

WHY: you cannot judge a release without knowing what shipped last time and what
is shipping now, and a dirty tree makes the verdict irreproducible.

1. Confirm a **clean working tree** on the release branch — `git status` shows
   no untracked or modified files.
2. **Read the last release report** (`tests/fleet/report.md`) so you know what
   was flagged previously and can diff against it.
3. **Read the release notes draft** (`RELEASE.md` / `CHANGELOG.md`) so you know
   the user-visible surface you are signing off.

### Phase 2 — Sanity gates (steps 4–10, fast, offline)

WHY: these are the cheap mechanical gates (`tests/sanity/RULES.md`)
that must pass in seconds before any slow work; they gate every PR, not just
releases.

4. **Formatting + lint:** `cargo fmt --check && cargo clippy -- -D warnings`.
   Pass = zero warnings.
5. **Rule-engine unit tests:** `cargo test`. Pass = all rule tests green.
6. **Structural validation:** `cargo test --test check structure`. Pass = all
   benchmarks + agents have required files and labels present.
7. **Compose config parse:** `cargo test --test check compose`. Pass = every
   `benchmarks/*/compose.yaml` parses via `docker compose config`.
8. **Dockerfile health catalog:** `cargo test --test check dockerfile`. Pass =
   every Dockerfile green, zero red.
9. **Trajectory health catalog:** `cargo test --test check trajectory`. Pass =
   every fixture green.
10. **Count reconciliation:** `cargo test --test check counts`. Pass =
    benchmark / agent / model counts match README claims.

### Phase 3 — Build sweep (steps 11–14, slow)

WHY: an image that does not build cannot be released; core images must build
first because benchmarks `COPY --from=` them
(`tests/build/RULES.md:3`).

11. **Core images:** `cargo test --test build core -- --ignored`. Pass = every
    core image (entrypoint, test-exact-match, litellm, llm-bridge) builds.
12. **Benchmark images:** `cargo test --test build benchmarks -- --ignored`.
    Pass = all build, or the known-failing list diffs cleanly against the prior
    run and `tests/build/known-broken.md`.
13. **Agent images:** `cargo test --test build agents -- --ignored`. Pass = all
    agents build.
14. **Model images:** `cargo test --test build models -- --ignored`. Pass = all
    models build.

### Phase 4 — Replay and end-to-end (steps 15–17)

WHY: replay proves the recorded fixtures still reproduce
(`tests/replay/RULES.md`); the live smoke proves a fresh run
works end-to-end (`tests/live/RULES.md`).

15. **Replay every fixture:** `cargo test --test replay -- --ignored`. Pass =
    every trajectory reproduces the same score. Fixtures in
    `tests/replay/fixtures/broken.json` are informational, not blocking.
16. **One fresh live smoke:** `cargo test --test run smoke -- --ignored`. Pass =
    a score file exists with score in [0, 1].
17. **Eyeball the smoke trajectory** — read `/tmp/eval-smoke/trajectory.jsonl`.
    Pass = the agent saw a real task and the response looks sane (this is the
    manual half of the end-to-end check).

### Phase 5 — Upstream + security (steps 18–22)

WHY: a pinned reference that has rotted, or a leaked secret, silently breaks the
release; these run only at release time because they need the network
(`tests/upstream/RULES.md:1`).

18. **Dataset revisions resolve:** `cargo test --test upstream datasets -- --ignored`.
    Pass = no 404s on HuggingFace / GitHub raw URLs. Any 404 is red; a 401/403
    auth wall is yellow and graduates to `tests/build/known-broken.md`.
19. **Pinned pip / npm versions exist:** `cargo test --test upstream packages -- --ignored`.
    Pass = no yanked or removed versions.
20. **`FROM` base images pullable:** `cargo test --test upstream bases -- --ignored`.
    Pass = no dangling base refs.
21. **hadolint scan (optional external linter):** `hadolint $(find . -name Dockerfile)`.
    Pass = zero errors; review warnings.
22. **Secret scan:** `gitleaks detect --source . --no-git`. Pass = zero
    findings.

### Phase 6 — Procedural audits (steps 23–27)

WHY: these catch what is wrong but passes the mechanical rules — the gap
mechanical rules cannot reach. They are toolchain-agnostic: a human walks them
in an editor, or a sub-agent walks them in batch; both produce the same report
shape so findings diff cleanly across releases.

23. **Walk new or changed Dockerfiles** — run the `audit-dockerfile` skill
    (7 questions per file) over each new/changed `Dockerfile`. Output: yes / no
    / n.a. with a one-line reason per question.
24. **Walk new or changed fixtures** — run the `audit-trajectory` skill (5 task
    + 5 run questions per fixture) over each new/changed
    `*.trajectory.jsonl`. Output: a written verdict per fixture.
25. **Walk the fleet** — run the `audit-fleet` skill (the 10 release questions,
    whole repo). Output: 10 answers and a red / yellow / green verdict.
26. **Agent roster still representative** — eyeball `agents/` against the current
    state of the art. Note any gaps.
27. **Benchmark set still representative** — eyeball `benchmarks/` against recent
    arxiv + leaderboards. Note any gaps.

### Phase 7 — Docs (steps 28–32)

WHY: a release whose README does not work from a clean clone, or whose RULES
have drifted from the code, ships a broken promise.

28. **README Quick Start from a clean clone** — fresh clone, follow the README
    verbatim. Pass = runs end-to-end without edits.
28a. **`docs/` sufficient and compliant** — every user-visible change in this
    release is reachable from `docs/` (no user-facing knowledge left only in
    source/commits/heads) and the affected pages comply with
    [`doctrine/docs/RULES.md`](../../docs/RULES.md). Pass = no gap a user could
    hit, links resolve.
29. **RULES still match the repo** — run the `audit-rules-drift` skill against
    recent commits. Pass = no drift (or drift documented).
30. **Every benchmark has a README:** `cargo test --test check benchmark_readmes`.
31. **Every agent has a README:** `cargo test --test check agent_readmes`.
32. **CHANGELOG entry written** — edit `CHANGELOG.md`. Pass = one bullet per
    user-visible change.

### Phase 8 — CI (steps 33–34)

WHY: local green is not enough; the published artifact must come from a green CI
on `main`.

33. **CI green on `main`** — `gh pr checks` / GitHub Actions. Pass = all
    workflows green.
34. **Release workflow ran recently** — check the last run timestamp. Pass =
    within 7 days.

### Phase 9 — Fleet report (steps 35–37)

WHY: the report is the single artifact a release manager reads to classify the
verdict; it MUST be regenerated mechanically, never hand-edited between runs
(`tests/fleet/RULES.md:7`).

35. **Generate the mechanical half:** `cargo test --test fleet -- --ignored`.
    Produces the auto section of `tests/fleet/report.md`.
36. **Paste the audit answers** from steps 23–27 into the manual section of the
    report.
37. **Classify the overall verdict** — apply the fleet classification
    (`tests/fleet/RULES.md:4`): green = every gate and audit
    green; yellow = some yellows, no reds; red = any red. Red is not
    ship-ready.

### Phase 10 — Release (steps 38–43)

WHY: tagging, publishing, and verifying the published artifacts is the act of
shipping; the report is committed under the tag so the release carries its own
verification record (`tests/fleet/RULES.md:8`).

38. **Tag the commit:** `git tag -s eval-vX.Y.Z` (signed). *(Do not run git in a
    dry verification; this is the live release step.)*
39. **Push the tag + trigger the release workflow:** `git push origin eval-vX.Y.Z`.
40. **Verify images published** — `docker pull quay.io/eval-containers/<image>:eval-vX.Y.Z`
    for each. Pass = every expected tag exists.
41. **Verify signatures / attestations** — `cosign verify quay.io/eval-containers/<image>:eval-vX.Y.Z`.
42. **Smoke test one image from a clean machine** — pull + run from a different
    host. Pass = end-to-end works from nothing.
43. **Attach the report to the GitHub release** —
    `gh release create ... --notes-file tests/fleet/report.md`, and commit the
    final `report.md` alongside the tag.

### Phase 11 — Post-release (steps 44–46)

WHY: a release is not done until gaps are tracked and the report is archived for
the next diff.

44. **Announce the release** (Slack / Discord / site) with the release notes.
45. **File follow-up issues for every yellow finding** — `gh issue create` per
    yellow, so every known gap has an issue.
46. **Archive this run's `report.md` with the tag** so the next release can diff
    against it.

## Executor and frequency summary

- **`cargo test` (machine):** steps 4–16, 18–20, 30, 31, 35.
- **External tools:** step 21 (hadolint), step 22 (gitleaks).
- **Procedural audit (human or sub-agent):** steps 23–25 via the audit skills.
- **Human only:** steps 1–3, 17, 26–29, 32–34, 36–46.

Frequency: steps 4–10 run every commit; add 11–14 every PR; **all 46** every
release; steps 18–20 and 23–29 also run on quarterly drift sweeps.

## When to run

- Before cutting any release tag (mandatory — walk all 46 steps).
- The contribution-verification subset (steps 4–15, no audits, no live, no
  network) on every PR.

## References

- `doctrine/verification/RULES.md` — the two verification processes and the
  precedence rule this walk implements.
- The four audit skills this walk invokes: `audit-dockerfile`,
  `audit-trajectory`, `audit-fleet`, `audit-rules-drift`.
- Per-category rules: `doctrine/verification/{sanity,build,replay,upstream,live,fleet}/RULES.md`.
