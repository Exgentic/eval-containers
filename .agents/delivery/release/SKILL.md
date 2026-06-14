---
name: release
description: >-
  Cut a release of the Eval Containers image fleet — pass the readiness
  gate, then build, tag, and push every benchmark, agent, model, gateway,
  and core image to the registry in bulk. Use when tagging a release or
  pushing the fleet to ghcr.io/exgentic. For building images during
  local dev (one artifact at a time, no push, no readiness gate), use the
  `build` skill instead; this skill is the full release flow that wraps it.
---

# Release the image fleet

Releasing means producing the whole fleet — every benchmark, agent,
model, gateway, and core image — tagged, labeled, and pushed to
`ghcr.io/exgentic`. The guiding principle: **CI builds the fleet;
humans build one thing at a time** (see the `build` skill for the
single-artifact loop). A release is the one time the entire fleet builds
and ships together, so it MUST pass the full readiness gate first.

A release ships the **CLI at the same version** too: the `vX.Y.Z` tag also
fires `.github/workflows/release.yml` (cargo-dist), which builds the
`eval-containers` binaries and publishes the crate. This skill covers the
fleet half; the unified outcomes — one tag, and which workflow owns what —
are [`.agents/delivery/RULES.md`](../RULES.md). Because the tag *is* the
version (`.agents/RULES.md:9`), bump `Cargo.toml`,
`benchmarks/_chart/Chart.yaml`, and `CHANGELOG.md` to the release version
**before** tagging, or the release aborts on the version-agreement gate
(`.agents/delivery/RULES.md:6`). Curate `CHANGELOG.md`'s `[Unreleased]`
section here — drawn from the commit log, in the Keep a Changelog sections,
for consumer-visible changes only; it is release-owner-curated, never edited
per PR (`.agents/delivery/RULES.md:8`–`10`).

Serves: `.agents/RULES.md:1` (the image is the product),
`.agents/RULES.md:2` (standalone artifacts),
`.agents/RULES.md:14` (verification is normative — no release ships
red), and `.agents/RULES.md:15` (the bake graph is the build artifact).

## Steps

1. **Confirm release verification has run, not just contribution
   verification.** A release MUST pass every contribution gate *plus*
   the live fleet sweep, the procedural audits, and the upstream
   reachability check. These are two different processes triggered at
   different points — never conflate them
   (`.agents/verification/RULES.md:1`, `.agents/verification/RULES.md:2`).
   Why: contribution gates run offline on every PR; the release-only
   gates (live, upstream, audits) are what certify the fleet actually
   produces trajectories end-to-end against a real model.

2. **Run the release-only gates out of band first.** The fleet report
   is a pure aggregator — it probes each category's log, it does not
   re-run them (`tests/fleet/RULES.md:1`,
   `tests/fleet/RULES.md:2`). So run, in any order:
   - `cargo test --test build -- --ignored` — the build sweep
     (`tests/build/RULES.md`). On macOS/podman set
     `DOCKER_HOST` to the podman machine socket
     (`tests/build/RULES.md:9`); CI uses the default.
   - `cargo test --test upstream -- --ignored` — every pinned `FROM`,
     `upstream_base` label, and HF/GitHub URL still resolves
     (`tests/upstream/RULES.md`). Any 404 is red
     (`tests/upstream/RULES.md:9`).
   - `cargo test --test live -- --ignored` — the live fleet sweep:
     every buildable benchmark, ≥3 tasks each, against the model of
     record (gpt-5.4) with the reference agent (claude-code)
     (`tests/live/RULES.md:3`,
     `tests/live/RULES.md:4`,
     `tests/live/RULES.md:5`). Respect
     `EVAL_LIVE_BUDGET_USD` (`tests/live/RULES.md:7`).
   Why first: the fleet report reads these logs; stale logs yield a
   stale verdict.

3. **Render the fleet report and read the verdict.** Run
   `cargo test --test fleet -- --ignored` to regenerate
   `.agents/verification/fleet/report.md` from scratch — never
   hand-edit it (`tests/fleet/RULES.md:7`). The verdict
   is red / yellow / green (`tests/fleet/RULES.md:4`).
   **No release MAY ship with a red verdict** (`.agents/RULES.md:14`).

4. **Walk the go/no-go readiness checklist.** Confirm every gate is
   green or justified-yellow, and that each outstanding yellow has a
   documented root cause and a reason it is not release-blocking. The
   full checklist — verdict classification, gate matrix, and
   outstanding-findings policy — is in
   [references/readiness.md](references/readiness.md). Why: a green
   report is necessary but not sufficient; a yellow ships only when the
   gaps are enumerated and understood.

5. **Promote any new live-sweep trajectories to fixtures.** Each passing
   live run becomes a replay fixture so contribution verification can
   re-run it at zero cost (`tests/live/RULES.md:2`,
   `tests/live/RULES.md:11`). Rename
   `output/<bench>/<task>/model/trajectory.jsonl` →
   `.agents/verification/replay/fixtures/<bench>-<task>-claude-code.trajectory.jsonl`
   and add a `provenance.json` entry recording model, agent version,
   benchmark data_revision, timestamp, and the release tag
   (`tests/replay/RULES.md:6`). A run that fails an
   inspection rule is NOT promoted — record it in
   `.agents/verification/live/known-broken.md` instead
   (`tests/live/RULES.md:12`). Why: fixtures are the
   immutable ground truth that makes the next cycle's offline replay
   meaningful (`tests/replay/RULES.md:4`).

6. **Build and push the fleet via Docker Bake.** The build graph is an
   artifact in the tree: every `core/`, `agents/`, `benchmarks/`,
   `models/`, `gateways/` directory ships a `docker-bake.hcl` next to
   its Dockerfile (`.agents/RULES.md:15`). Release builds go through
   `docker buildx bake` — the same canonical invocation tests and the
   CLI use. Tag the fleet via the fleet-wide `TAG` variable
   (`.agents/RULES.md:15`, sub-rule b) and push:

   ```bash
   # Build + push the whole fleet at the release tag (the actual release step)
   TAG=v1.2.0 docker buildx bake --push

   # Override the registry for a staging push
   REGISTRY=ghcr.io/eval-containers TAG=v1.2.0 docker buildx bake --push
   ```

   `REGISTRY` (default `ghcr.io/exgentic`) and `TAG` (default
   `latest`) are declared once at the repo root `./docker-bake.hcl` and
   picked up by auto-discovery (`.agents/RULES.md:15`, sub-rules b–c).
   The `build` skill covers composing and building individual targets,
   groups, and eval combinations; this step is the fleet-wide push.
   Why bake: it is Docker's native declarative multi-image build, and
   keeping every consumer (CI, CLI, tests, OC translators) on one
   invocation is what makes the build graph reusable
   (`.agents/RULES.md:15`, sub-rule d).

7. **Prefer letting CI build the fleet.** `.github/workflows/release-images.yml`
   runs bake on every push to `main` (tag: `latest`) and every `v*` tag
   (tag: the git tag), setting `GIT_SHA` and `BUILD_DATE`, then
   `bake --push`es the result. CI runs on real Docker on Linux, where
   the full sweep is clean; local podman-on-macOS chokes the parallel
   fleet build on network contention (a documented, non-structural
   caveat — see [references/readiness.md](references/readiness.md) and
   `.agents/verification/build/known-broken.md`). Why: humans building
   100+ images locally is slow and flaky; CI is the authoritative fleet
   builder.

8. **Commit the fleet report alongside the release tag.** When cutting
   the tag, commit the final `.agents/verification/fleet/report.md` so
   the release artifact carries its own verification record
   (`tests/fleet/RULES.md:8`). Why: a release that ships
   without its certifying report cannot be audited after the fact.

## Releasing the CLI alongside the fleet

The CLI half is automatic on the tag — no bake, no fleet gate. Once
`Cargo.toml`, `benchmarks/_chart/Chart.yaml`, and `CHANGELOG.md` carry the
release version and the `vX.Y.Z` tag is pushed,
`.github/workflows/release.yml` (cargo-dist) builds the cross-platform
`eval-containers` binaries, generates the installers, publishes the crate to
crates.io, and creates the tag's GitHub Release
(`.agents/delivery/RULES.md:2`, `.agents/delivery/RULES.md:4`). The fleet
workflow (`release-images.yml`) publishes only to the registry and MUST NOT
touch that GitHub Release, so the two halves compose on one tag instead of
clobbering each other (`.agents/delivery/RULES.md:3`–`4`). crates.io
versions are immutable — never reuse a number, so the first unified tag after
a manual crate publish starts one patch above it
(`.agents/delivery/RULES.md:7`).

## What this skill does NOT cover

- **Building one artifact for local dev** — that's the `build` skill.
- **Running tests or agents** — bake only builds. Use
  `cargo test --test replay` / `--test live`, or `docker compose up` /
  `eval-containers run`.
- **Verifying labels post-build** — that's the build sweep
  (`tests/build/RULES.md`).

## References

- [Delivery rules](../RULES.md) — the version/release outcomes (one tag,
  workflow ownership, guards) this procedure satisfies.
- [references/readiness.md](references/readiness.md) — the go/no-go gate.
- [Docker Bake docs](https://docs.docker.com/build/bake/)
- The `build` skill (`.agents/build/SKILL.md`) — single-artifact and
  eval-combination builds.
