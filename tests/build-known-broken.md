# Known-broken benchmark builds

The local build sweep (`cargo test --test build build_every_benchmark -- --ignored`) attempts to build every benchmark image from scratch. 81 of 96 succeed cleanly. 5 skip cleanly (per-task-build pattern, see `tests/build.rs::is_per_task_benchmark`). The 10 below fail on the dev laptop, but **9 of them pass on the GitHub Actions `ubuntu-latest` x86_64 runner** that `.github/workflows/build-sweep.yml` uses for CI. They are documented here so a local sweep operator can diff their failure set against this list before opening a ticket.

This list is tracked by commit. Before each release, re-run the sweep on CI and update the first section below.

## Current status (sweep round 4, 2026-04-15, 2582s wall)

```
81/96 pass · 5 skip (per-task-build) · 10 fail
```

## Platform-only failures (pass on x86_64 CI)

These fail under `qemu-user-static` emulation on an arm64 host (Apple Silicon laptop running podman). The root cause is a native dependency that segfaults or aborts when run under QEMU. The failures are reproducible: retrying the same commit on the same host always fails at the same step. The failures are also consistently absent on `ubuntu-latest` runners.

| Benchmark | Upstream cause | Local failure |
|---|---|---|
| `appworld` | `pyarrow` 18.1.0 wheel segfaults under qemu inside `ghcr.io/stonybrooknlp/appworld:latest` (itself x86_64-only) | `qemu: uncaught target signal 11 (Segmentation fault)` during `RUN python3 <<PYEOF` |
| `swe-bench` | `swebench==3.0.18` pip install pulls native deps that abort under qemu; base image is also x86_64-only | `exit 1` during `pip install --target /tests/deps swebench==3.0.18` |

## Upstream data-reachability failures (credential- or auth-gated)

These benchmarks need credentials or access to gated datasets that aren't available at `docker build` time on an unauthenticated host. CI passes them because the CI runner has `HF_TOKEN` configured via GitHub secrets; local laptops usually don't.

| Benchmark | Upstream | Gate |
|---|---|---|
| `flores200` | `dl.fbaipublicfiles.com/nllb/flores200_dataset.tar.gz` | Meta hosting dropped / requires signup form |
| `gaia` | `huggingface.co/datasets/gaia-benchmark/GAIA` | `HF_TOKEN` required for gated dataset |
| `hle` | `huggingface.co/datasets/cais/hle` | `HF_TOKEN` required |
| `mt-bench` | `huggingface.co/datasets/lmsys/mt_bench_human_judgments` | `HF_TOKEN` required |
| `osworld` | upstream Python package install | packaging issue or network |
| `realworldqa` | `huggingface.co/datasets/xai-org/RealWorldQA` | `HF_TOKEN` required |
| `workarena` | upstream GitHub raw | transient / rate-limit |
| `frontiermath` | private Epoch dataset | not publicly reachable |

## What to do about them

- **Before release**: run the full sweep on CI (`.github/workflows/build-sweep.yml`, manual dispatch). The 2 platform-only failures should pass there. The 8 auth-gated ones should also pass once CI secrets are wired for the relevant providers.
- **On a fresh laptop**: if your failure set matches this list, nothing is wrong with your branch. If it's a superset, diff and investigate the new entries.
- **If one of these starts passing locally**: celebrate, and delete it from this file in the same commit.

## Cross-reference

- `tests/fixtures/broken.json` — broken trajectory fixtures (different axis: run-time, not build-time).
- `tests/VERIFY.md` step 12-13 — where the build sweep lives in the release checklist.
- `.github/workflows/build-sweep.yml` — the CI sweep that is the authoritative green/red for these benchmarks.
