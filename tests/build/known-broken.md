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

`81/96 pass · 5 skip (per-task-build) · 10 fail` — sweep round 4, 2026-04-15, 2582s wall (before the Rosetta + `swebench` pin fixes below landed). The next sweep will reflect those fixes.

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

## Per-task-build benchmarks needing private GHCR access

These 7 benchmarks have `FROM ghcr.io/<upstream>/<name>.${DOCK_TASK_ID}:...` in their Dockerfiles — they pull per-task base images from private GHCR packages that return 401 anonymously. Running them requires authenticated `GHCR_TOKEN` with access to the respective organizations, or a local mirror of the upstream per-task images.

| Benchmark | Upstream registry | Gate |
|---|---|---|
| `swe-bench` | `ghcr.io/epoch-research/swe-bench.eval.x86_64.*` | 401 — needs Epoch Research access |
| `swe-bench-pro` | `ghcr.io/swe-bench/swe-bench-pro.eval.x86_64.*` | 401 — needs SWE-Bench access |
| `swe-lancer` | `ghcr.io/openai/swelancer.*` | 401 — OpenAI private registry |
| `mle-bench` | `ghcr.io/openai/mle-bench.*` | 401 — OpenAI private registry |
| `terminal-bench` | `ghcr.io/laude-institute/terminal-bench/*` | 401 — needs Laude Institute access |
| `cybench` | `ghcr.io/andyzorigin/cybench.*` | Per-task images not yet published upstream |
| `compilebench` | builds locally; transient apt issues under load | — |

The sweep driver adds curated representative task IDs (`tests/live/test.rs::per_task_representative`) but cannot complete a green run until credentials land in the release-runner secrets. This is a deployment prerequisite, not a code bug.

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
