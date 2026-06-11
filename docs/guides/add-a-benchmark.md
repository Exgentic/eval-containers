# Add a benchmark

*Guide · for contributors · the canonical procedure is [`.agents/benchmarks/add-benchmark/SKILL.md`](../../.agents/benchmarks/add-benchmark/SKILL.md).*

Adding a benchmark is governed by doctrine. This page is a map, not a
replacement — follow the skill and the rules it links.

1. **Read the rules** — [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md)
   (what a finished benchmark must be).
2. **Follow the skill** — [`.agents/benchmarks/add-benchmark/SKILL.md`](../../.agents/benchmarks/add-benchmark/SKILL.md)
   (the step-by-step procedure, with a template).
3. **Ship the deploy artifacts** — every benchmark needs `container.Dockerfile`
   and `compose.yaml` (the single-container and compose surfaces;
   `tests/sanity/check.rs` enforces it). The k8s surface is the shared chart
   selected with `--set benchmark=<x>` — add a `benchmarks/_chart/presets/<x>.yaml`
   only if the benchmark needs bespoke topology. See
   [Triple-mode](../concepts/triple-mode.md).
4. **Open the PR** using the
   [benchmark PR template](../../.github/PULL_REQUEST_TEMPLATE/benchmark.md),
   which lists every required label, env var, and evidence step.

For the conventions that keep images thin and bake files valid, see
[`.agents/delivery/build/SKILL.md`](../../.agents/delivery/build/SKILL.md).
