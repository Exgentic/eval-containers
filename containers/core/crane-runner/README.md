# crane-runner (`--mode crane`)

One generic image that materializes any `(benchmark, agent)` eval at **run time**
by fusing the per-axis images, instead of pulling a pre-built
`evals/<benchmark>--<agent>` combination image.

## Why

`compose` / `container` / `job` modes all run a pre-built combination image, so
the `evals/<b>--<a>` matrix (benchmarks × agents) must exist. Per-task benchmarks
(SWE-bench) make it worse — a different image per task, which a single
`images.benchmark` can't fan out. The crane runner pulls the per-axis
`benchmarks/<b>` + `agents/<a>` images at run time and fuses them in one
container, so:

- the combination matrix becomes an **optional pre-bake**, and
- per-task benchmarks run from **one** image (it pulls the per-task rootfs).

It is the single-container analog of the generic `compose` file — one better: it
composes the *axes* at run time instead of pulling a pre-fused product.

## How (daemonless — no DinD)

`materialize` does: **crane export** the benchmark rootfs (download + untar, no
daemon) → **overlay the agent** → **bwrap/chroot** in to run agent + grade. The
distinction from DinD: crane only *downloads a filesystem*; it never runs a
daemon. The core primitive (crane pull + run-in-rootfs + agent-edits-testbed) is
proven daemonless — run [`crane-poc.sh`](crane-poc.sh) in any Linux container.

## Status — first cut

- ✅ **Fusion** ([1]/[2] in `materialize`) — the proven primitive.
- ⛳ **Validation gate**: wiring the in-rootfs gateway/otel via `process-compose`
  and the root-only grader perms ([3]) reuses the combination image's existing
  `/usr/local/bin/run` mechanism; **end-to-end against real images is not yet
  run**. Pin pulls by **digest** for reproducibility before shipping.
- This is a **doctrine-level** addition (late- vs early-binding): it needs a rule
  for *when* crane applies (`per-task`, or opt-in) and the digest-pin requirement.

## Use (additive, opt-in — nothing else depends on it)

```bash
eval-containers run aime --agent codex --model openai/<model> --mode crane
# → docker run --rm -e EVAL_* -v output:/output <registry>/core/crane-runner:latest
```
