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
daemon) → **overlay the agent** → **chroot/bwrap** in to run agent + grade. The
distinction from DinD: crane only *downloads a filesystem*; it never runs a
daemon — so nothing here needs privilege or a Docker socket. Verified daemonless
against the real swe-bench + claude-code images (see the PR).

## Status — first cut (statically verified)

Proven:
- **Fusion** ([1]/[2]) against **real** images — the real swe-bench `/testbed` (sympy
  repo) ⊎ the real claude-code `/opt/agent` reproduce the combination image's layout.
- **Isolation survives the fusion**: `export` preserves root-only modes/owners
  (`/tasks` `0600`, `/tests` & `/opt/gateway` `0700`, root), so *extract as root* and
  run the agent through the existing `gosu agent` / `env -i` pipeline (which even hides
  the task id from the model) and the answers stay unreadable. The invariant is
  **reused, not reinvented**.

Build-out ([3]) — wire the existing runtime, don't invent one:
1. **Bake the fixed core** into this image: `otelcol`, `process-compose`, `gosu`,
   `/usr/local/bin/{run,write-result}` + configs.
2. **Pull the 3rd axis** (`models/<gateway>` → `/opt/gateway`) alongside benchmark+agent;
   run the agent image's `install.sh` (symlinks).
3. **Extract as root**, then `exec /entrypoint.sh /usr/local/bin/run` — the existing
   5-process pipeline (otelcol → gateway → agent → verifier → result).

Hard constraint: **arch** — the runner, the node, and the pulled rootfs must match
(swe-bench is `x86_64`; the runner built here is `arm64`). Plus **digest-pinned** pulls,
a **conformance test** (`crane(X) == container(X)`), a **bake target**, and a **doctrine
rule** (when crane applies + the digest-pin + isolation invariants).

## Use (additive, opt-in — nothing else depends on it)

```bash
eval-containers run aime --agent codex --model openai/<model> --mode crane
# → docker run --rm -e EVAL_* -v output:/output <registry>/core/crane-runner:latest
```
