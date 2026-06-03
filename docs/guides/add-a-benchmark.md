# Add a benchmark

*Guide · for contributors · the canonical procedure is [`doctrine/benchmarks/add-benchmark/SKILL.md`](../../doctrine/benchmarks/add-benchmark/SKILL.md).*

Adding a benchmark is governed by doctrine. This page is a map, not a
replacement — follow the skill and the rules it links.

1. **Read the rules** — [`doctrine/benchmarks/RULES.md`](../../doctrine/benchmarks/RULES.md)
   (what a finished benchmark must be).
2. **Follow the skill** — [`doctrine/benchmarks/add-benchmark/SKILL.md`](../../doctrine/benchmarks/add-benchmark/SKILL.md)
   (the step-by-step procedure, with a template).
3. **Ship the three deploy artifacts** — every benchmark needs all of
   `container.Dockerfile`, `compose.yaml`, and `values.yaml`
   (the triple-mode invariant; see [Triple-mode](../concepts/triple-mode.md)).
   `tests/sanity/check.rs` enforces it.
4. **Open the PR** using the
   [benchmark PR template](../../.github/PULL_REQUEST_TEMPLATE/benchmark.md),
   which lists every required label, env var, and evidence step.

For the conventions that keep images thin and bake files valid, see
[`doctrine/delivery/build/SKILL.md`](../../doctrine/delivery/build/SKILL.md).
