---
benchmark: zerobench-agent
host: local podman+Rosetta
commit: feat/add-zerobench
---
# Audit — zerobench-agent

`✓` verified · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | benchmark image builds (gated HF fetch via HF_TOKEN secret) |
| running | ? | agent sweep in progress (Claude Code × Opus 5); not finalized |
| grader-shared | ✓ | uses `core/test-answer-match` (same scorer the native `zerobench` imports) — module + CLI unit-tested, fail-closed |
| isolation | ? | not audited per-benchmark |
| oracle | ✓ | `solution.sh` emits the gold behind the `FINAL ANSWER:` sentinel; `eval-containers oracle zerobench-agent --task-id 0 --local` scores gold=1.0, no-op=0.0 (so the grader accepts a correct answer and rejects an absent one) |
| replicate-official | ? | pending vs the leaderboard's tool-assisted entries |

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
