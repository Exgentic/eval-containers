---
name: publish
description: >-
  Publish fleet images on demand — a benchmark, an agent's combos, single
  tasks, or the whole fleet — by dispatching the release workflow with the
  right scope, then confirm what landed and under which hash tag. Use when
  something merged to main is needed before tonight's nightly channel run,
  or to preview what a change would build. For a versioned vX.Y.Z release
  use the `release` skill; to build an image locally, `build`.
---

# Publish on demand

The `latest` channel is nightly (`.agents/delivery/RULES.md:16`). Between
nights, the same workflow publishes any subset on request: every run
publishes only images whose inputs changed, under their hash tag first and
`latest` after (`.agents/delivery/RULES.md:18`–`:20`), so a dispatch is
idempotent and safe to repeat.

Serves: `.agents/delivery/RULES.md:5` (SemVer only by tag or explicit
dispatch), `:16` (nightly channel), `:18`–`:20` (hash tag, immutability,
`latest` alias).

1. **Confirm the change is on `main`, and dispatch from `main`.** A dispatch
   from a branch publishes under the same `:latest` names the nightly uses.
   Why: `latest` is a snapshot of a `main` tip (rule 20); a branch preview
   under hash tags only is a planned follow-up, not something to improvise.

2. **Name the scope.** Leaf targets are the first column of
   `containers/scripts/fleet-hash.sh graph`. Choose `only` (leaf targets),
   `combo_agents` (agents whose `evals/<benchmark>--<agent>` combos to build
   over the benchmarks in scope; blank = none), `tasks` (task ids of a
   per-task benchmark named in `only`), and whether to keep
   `include_per_task` / `include_standalone` on. Why: the enumerate job
   intersects every matrix with `only`, so an unnamed leaf is not built,
   and combos are opt-in.

3. **Preview with `dry_run=true` when unsure.** It prints `bake --print` per
   target and pushes nothing.

4. **Dispatch and watch.**

   ```bash
   gh workflow run release-images.yml --ref main -f only="benchmark-gsm8k agent-claude-code" -f combo_agents=claude-code -f include_per_task=false
   gh run watch "$(gh run list --workflow=release-images.yml --limit 1 --json databaseId --jq '.[0].databaseId')"
   ```

   Never pass `tag`: that publishes a versioned release (rule 5), not a
   preview.

5. **Read the result.** The `report` job's summary lists the leaf images in
   scope with build time and size; each build job's log says `fresh`
   (already published at its hash, nothing to do), `carried forward`, or
   shows the build. Why: "nothing built" is the correct outcome for an
   unchanged input, not a failure.

6. **Record the hash tag** for anyone who will pin the result:

   ```bash
   containers/scripts/fleet-status.sh hash ghcr.io/exgentic/benchmarks/gsm8k:latest
   ```

   used as `EVAL_BENCHMARK_TAG=<hash>` (`EVAL_AGENT_TAG`, `EVAL_GATEWAY_TAG`
   likewise). Why: `latest` moves tonight; a hash tag never does (rule 19).
