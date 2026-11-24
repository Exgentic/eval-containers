---
benchmark: assertllm2-verify-nospec
host: local Rancher Desktop (macOS, linux/arm64, native)
commit: feat/751-assertllm2-verify
---
# Audit — assertllm2-verify-nospec (AssertLLM2, 3 designs / 19 tasks, no spec)

Status: `✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker buildx build` of `core/assertllm2-harness` and this image on linux/arm64; the harness's build runs `build-tasks.py` and `check-tasks.py` and fails unless the tasks that pass are exactly `tasks.txt` (19). Not yet built for linux/amd64 |
| running | ✓ | one live run: task 1 (`ima_adpcm_encoder/M_0004`) with `claude-code` and `aws/claude-opus-5-5` through `eval-containers run --local` (compose): agent exit 0 in 54 s, answer valid, graded. One task only |
| isolation | ✓ | in the built image: `/app` holds only `rtl/` for task 0 and task 7 (no `spec.md`, no `figures/`; `assertllm2-verify` on the same tasks still has both). As uid 1002 (`setpriv`): `cat /tasks/0/spec.txt`, `ls /tasks`, `cat /tasks/all.jsonl`, `ls /tasks/assets` all denied, and `find / -xdev` for `spec.md`, `spec.txt` or `*figures*` finds nothing. The prompt names the top module, says no specification is provided, and leaves no placeholder unfilled. The task files themselves are `assertllm2-verify`'s |
| oracle | ? | not run for this setup (`solution.sh` is `assertllm2-verify`'s, byte-identical; it reads no spec). `assertllm2-verify`'s oracle runs do not count here |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | n/a | upstream publishes assertion-generation scores (JasperGold), not verdicts; nothing to replicate |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `eval.benchmark.internet="false"` and `ENV EVAL_INTERNET=false`; `compose.yaml` extends the shared runner (internal network only); grading is offline |
| agent-nonroot | ? | the shared runner and `entrypoint.sh` are `assertllm2-verify`'s, but the symlink check was not repeated on this image |
| secrets-isolated | ✓ | no `ARG`/`ENV` secret in this Dockerfile or the harness's; no credential at build |
| resource-limited | ✓ | the shared runner default, 2 CPU / 2G (`compose/runner.yaml`) and the chart default 2 / 2Gi — declared, not measured |

## Size

| Metric | Value |
|--------|-------|
| image | 364 MB (`docker image ls`; the same as the harness: labels and ENV only) |
| per-task multiplier | 1 (shared-env) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? (not timed; this image adds labels and ENV on top of the harness) |
| grade | under 1 s (`grade.py`, stdlib) |
| end-to-end | 54 s agent time for one task (task 1, Opus 5.5) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (not measured) |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not published to ghcr.io/exgentic/benchmarks (new in this PR) |
| pull size | — | not published |

Tools in the image: no simulator or formal tool (`command -v iverilog verilator yosys sby z3 g++` finds none).
