# Known-broken benchmark builds

The local build sweep (`cargo test --test build build_every_benchmark -- --ignored`) attempts to build every benchmark image from scratch. Most succeed cleanly. A handful need credentials the local host doesn't have. This file documents them so a local operator can diff their failure set against it before opening a ticket.

This list is tracked by commit. Update the status snapshot below with every release sweep.

## Prerequisites for a full local sweep on Apple Silicon

1. **Podman with Rosetta** — x86_64 benchmarks use Rosetta native translation, not qemu. ~10× faster, no pyarrow / numpy segfaults. Enable via `podman machine ssh "sudo touch /etc/containers/enable-rosetta"` then restart the machine. See [tests/LOCAL.md](../LOCAL.md) for the full setup.
2. **`HF_TOKEN` in `.env`** — required for HuggingFace gated datasets. Get one at https://huggingface.co/settings/tokens (read scope).
3. **Accepted gated-dataset licenses** on HuggingFace for:
   - https://huggingface.co/datasets/cais/hle
   - https://huggingface.co/datasets/gaia-benchmark/GAIA
   - https://huggingface.co/datasets/lmsys/mt_bench_human_judgments
   - https://huggingface.co/datasets/xai-org/RealWorldQA

With all three in place, 11 of the 13 benchmarks in the "upstream-gated" section below move to green locally. The remaining 2 (`flores200`, `frontiermath`) depend on upstream hosts that have no HuggingFace path.

## Current status

`81/96 pass · 5 skip (per-task-build) · 10 fail` — sweep round 4, 2026-04-15, 2582s wall (before the Rosetta + `swebench` pin fixes below landed). The next sweep will reflect those fixes.

## Upstream data-reachability failures

These benchmarks need credentials or network paths the local host doesn't have. With the `HF_TOKEN` + accepted licenses described above, 6 of the 8 become runnable.

| Benchmark | Upstream | Gate | HF_TOKEN fixes? |
|---|---|---|---|
| `flores200` | `dl.fbaipublicfiles.com/nllb/flores200_dataset.tar.gz` | Meta pulled the anonymous link; requires their signup form | no |
| `gaia` | `huggingface.co/datasets/gaia-benchmark/GAIA` | Gated dataset | yes |
| `hle` | `huggingface.co/datasets/cais/hle` | Gated dataset | yes |
| `mt-bench` | `huggingface.co/datasets/lmsys/mt_bench_human_judgments` | Gated dataset | yes |
| `osworld` | upstream Python package install | transient packaging issue — needs revisit | maybe |
| `realworldqa` | `huggingface.co/datasets/xai-org/RealWorldQA` | Gated dataset | yes |
| `workarena` | upstream GitHub raw | transient / rate-limit — needs revisit | maybe |
| `frontiermath` | private Epoch dataset | not publicly reachable | no |

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
