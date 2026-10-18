---
benchmark: zerobench-agentic
host: local podman+Rosetta
commit: feat/add-zerobench
---
# Audit — zerobench-agentic

`✓` verified · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | benchmark image builds (gated HF fetch via HF_TOKEN secret) |
| running | ? | agent sweep in progress (Claude Code × Opus 5); not finalized |
| grader-shared | ✓ | uses `core/test-answer-match` (same scorer the native `zerobench` imports) — module + CLI unit-tested, fail-closed |
| isolation | ? | not audited per-benchmark |
| oracle | ✓ | `solution.sh` emits the gold behind the `FINAL ANSWER:` sentinel; `eval-containers oracle zerobench-agentic --task-id 0 --local` scores gold=1.0, no-op=0.0 (so the grader accepts a correct answer and rejects an absent one) |
| replicate-official | ? | pending vs the leaderboard's tool-assisted entries |
| live-trajectory | ✗ | **blocks release.** Task 0, agent `claude-code`, model `gpt-5.5` (2026-10-18): the agent hangs. Two attempts, 900s and 2700s, both exit 124 (live/RULES.md rule 28 red). The longer ceiling did not help — in the 2700s run the agent started 10:19:22, made ~5 model calls (101,232 in / 1,761 out), its last call completed 10:20:13, then produced nothing for ~44min until the kill. `agent/stdout.log` and `agent/stderr.log` are both 0 bytes and `agent/launch.json` is `[]`, so it is not a slow task but a stall in the launch/startup path — note this benchmark's entrypoint rewrites `/app` images and `TASK` before `exec "$@"`. No promotable trajectory exists, so `released` stays `"false"` (RULES.md 21a). Investigate before retrying: a third run at a higher ceiling is not justified. |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `EVAL_INTERNET=false` (label + ENV agree) |
| agent-nonroot | ✓ | agent runs as non-root (ubuntu/uid 1000) |
| secrets-isolated | ✓ | HF_TOKEN is an ephemeral build secret, never baked (RULES.md 8a) |
| resource-limited | ? | inherits shared compose/chart limits |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
