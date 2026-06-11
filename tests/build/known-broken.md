# Known-broken benchmark builds

The local build sweep (`cargo test --test build build_every_benchmark -- --ignored`) attempts to build every benchmark image from scratch. Most succeed cleanly. A handful need credentials the local host doesn't have. This file documents them so a local operator can diff their failure set against it before opening a ticket.

This list is tracked by commit. Update the status snapshot below with every release sweep.

## Prerequisites for a full local sweep on Apple Silicon

1. **Podman with Rosetta** — x86_64 benchmarks use Rosetta native translation, not qemu. ~10× faster, no pyarrow / numpy segfaults. Enable via `podman machine ssh "sudo touch /etc/containers/enable-rosetta"` then restart the machine. See [tests/LOCAL.md](../LOCAL.md) for the full setup.
2. **`HF_TOKEN` in `.env`** — required for HuggingFace gated datasets. Get one at https://huggingface.co/settings/tokens (read scope).
3. **Accepted gated-dataset licenses** on HuggingFace for:
   - https://huggingface.co/datasets/cais/hle
   - https://huggingface.co/datasets/gaia-benchmark/GAIA
   - https://huggingface.co/datasets/openlanguagedata/flores_plus  (flores200 now reads from here)

With all three in place, 12 of the 13 benchmarks in the "upstream-gated" section below move to green locally. The remaining 1 (`frontiermath`) depends on the private Epoch AI dataset which has no HuggingFace path.

## Current status

Fleet is now **100 benchmarks × 20 agents** after the April 2026 expansion.
Last serial sweep (round 4, 2026-04-15 at 96 benchmarks) was
`81/96 pass · 5 skip · 10 fail`. A fresh sweep against the 100-benchmark
fleet is pending and lands with the next release snapshot.

**Local full-fleet caveat.** On podman, `EVAL_BUILD_PARALLEL >= 4`
saturates the VM's network stack and produces non-deterministic
`curl`/`pip`/`apt-get` timeout storms (observed 2026-04-17). For local
full-fleet sweeps use `EVAL_BUILD_PARALLEL=2` or fewer; CI under real
Docker runs them reliably at higher concurrency. See
[tests/LOCAL.md](../LOCAL.md) Level 2b.

## Upstream data-reachability failures

These benchmarks need credentials or network paths the local host doesn't have. With the `HF_TOKEN` + accepted licenses described above, 6 of the 8 become runnable.

| Benchmark | Upstream | Gate | HF_TOKEN fixes? |
|---|---|---|---|
| `flores200` | `huggingface.co/datasets/openlanguagedata/flores_plus` | License-gated on HF | yes |
| `gaia` | `huggingface.co/datasets/gaia-benchmark/GAIA` | License-gated dataset | yes |
| `hle` | `huggingface.co/datasets/cais/hle` | License-gated dataset | yes |
| `mt-bench` | `raw.githubusercontent.com/lm-sys/FastChat/...question.jsonl` | Open (FastChat Apache-2.0) — rebuilt, no gate | n/a |
| `osworld` | `raw.githubusercontent.com/xlang-ai/OSWorld/...` | Open — transient packaging issue only, URL is live | n/a |
| `realworldqa` | `huggingface.co/datasets/xai-org/RealworldQA` (parquet convert branch) | Anonymous via convert/parquet branch — no HF_TOKEN needed | n/a |
| `workarena` | upstream GitHub raw | Task-id paths fixed + retry loop added at v0.5.3 | n/a |
| `frontiermath` | `huggingface.co/datasets/epoch-ai/frontiermath` | Private repo (returns 401 even for logged-in users). Requires `HF_TOKEN` on an account explicitly granted access via `math_evals@epochai.org` — not just license acceptance. | no (private, not gated) |

## Per-task-build benchmarks

These benchmarks have `FROM …${EVAL_TASK_ID}…` in their Dockerfiles. The
build sweep drives each with a curated representative task id from
`per_task_build_args` in `tests/build/test.rs`. Some upstreams are
anonymously pullable; others require credentials or a locally-built
upstream image.

| Benchmark | Upstream base | Gate |
|---|---|---|
| `swe-bench` | `docker.io/swebench/sweb.eval.x86_64.<task_id>` | Public — but the HF instance id must be sanitized (`__` → `_1776_`) to match the published tag. Fixed in `tests/build/test.rs` (2026-04-18). |
| `terminal-bench` | `ghcr.io/laude-institute/t-bench/python-3-13:20250620` | Public — anonymously pullable. Previous 60s failures were transient. |
| `cybench` | `ubuntu:24.04` + upstream git clone | Public — builds from source. Retry loop added 2026-04-18. |
| `compilebench` | parametric `${BASE_IMAGE}` + curl fetch | Public — curl retry added 2026-04-18. |
| `swe-bench-pro` | `sbp/instance:<task_id>` | **Local-only** — needs a pre-built instance image from `scaleapi/SWE-bench_Pro-os` harness. Deployment prerequisite, not a code bug. |
| `swe-lancer` | `swelancer_x86:latest` | **Local-only** — needs a pre-built image from `openai/preparedness`. Deployment prerequisite. |
| `mle-bench` | `mlebench-env:latest` | **Local-only** — needs a pre-built image from `openai/mle-bench`. Deployment prerequisite. |

The "local-only" row entries cannot be fixed inside this repo; they
require either a CI step that pre-builds the upstream image, or an
ahead-of-time mirror in the release registry. See release runbook.

## Agents with known local-harness failures

| Agent | Root cause | Mitigation |
|---|---|---|
| `plandex` | Multi-stage Dockerfile combines `FROM plandexai/plandex-server:...` and `FROM ghcr.io/exgentic/core/agent-base-rust:latest`. testcontainers-rs / bollard / BuildKit can't resolve the second locally-tagged FROM when the first references a remote image — attempts to pull `ghcr.io/exgentic/core/agent-base-rust` from the registry and fails with `unauthorized`. Direct `docker build agents/plandex/` from the shell succeeds (BuildKit classic-image-store path). | Runs fine on CI Linux real Docker. Locally, `docker build -t ghcr.io/exgentic/agents/plandex:latest agents/plandex/` works. Not a structural defect — a podman-BuildKit multi-stage quirk. |

## Fixed since round 4

| Benchmark | Root cause | Fix |
|---|---|---|
| `appworld` | we were running under qemu-user-static; pyarrow segfaulted | Enable Rosetta on podman machine (see [tests/LOCAL.md](../LOCAL.md)) |
| `swe-bench` | `swebench==3.0.18` was yanked from PyPI; package jumps 3.0.17 → 4.0.0 | Bumped pin to `swebench==3.0.17` |

Both verified locally. Deleting from the "failures" section above once the next round confirms green.

## What to do about a failure

- **Your failure set matches this list**: nothing is wrong with your branch. Continue.
- **Your failure set is a superset**: diff against this file and investigate new entries.
- **A listed failure starts passing locally**: delete it from this file in the same commit. This file is ground truth.

## Cross-reference

- [tests/replay/fixtures/broken.json](../replay/fixtures/broken.json) — broken trajectory fixtures (run-time axis, not build-time).
- [tests/VERIFY.md](../VERIFY.md) steps 12-13 — where the build sweep lives in the release checklist.
- [tests/LOCAL.md](../LOCAL.md) — local podman + Rosetta + BuildKit GC setup.
- [.github/workflows/build-sweep.yml](../../.github/workflows/build-sweep.yml) — the CI sweep that is the authoritative green/red for these benchmarks.
