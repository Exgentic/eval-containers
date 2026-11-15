# hle

**Status:** Legacy — not released (`eval.benchmark.released="false"`). Superseded
by HLE-Diamond, which is being added as a separate benchmark
([Exgentic/eval-containers#733](https://github.com/Exgentic/eval-containers/issues/733)).
Sample trajectory: [`tests/fixtures/hle-0-claude-code.traces.jsonl`](../../tests/fixtures/hle-0-claude-code.traces.jsonl)

HLE - Humanity's Last Exam (original 2,500-question set).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2500 |
| Environment | shared-env |
| Internet required | false |
| Released | no (legacy) |
| Upstream | [https://github.com/centerforaisafety/hle](https://github.com/centerforaisafety/hle) |
| Paper | [paper](https://arxiv.org/abs/2501.14249) |
| Dataset revision | `5a81a4c7271a2a2a312b9a690f0c2fde837e4c29` |

## Not comparable to official HLE scores

This benchmark keeps its original grading on purpose, so that existing runs stay
comparable with each other. It differs from the official protocol in three ways:

- **Grading.** Whitespace-normalised exact match against the reference answer.
  Official HLE uses an LLM judge that extracts the final answer and checks
  equivalence, so an answer like `0.5` for a reference of `1/2` scores 0 here and
  1 there. Scores here are a **lower bound** on judge-graded accuracy.
- **Prompt.** The agent is asked for only the final answer, not the official
  Explanation / Answer / Confidence format, so calibration error is not measured.
- **Judge availability.** The official default judge, `o3-mini-2025-01-31`, was
  shut down on 2026-10-23, so the published protocol cannot be reproduced as
  written. Artificial Analysis now uses a different judge and the HLE-Diamond set.

Do not report these numbers next to published HLE or Artificial Analysis scores.

## What the agent sees

The agent receives a task of the form: "$PROMPT" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match, after stripping all whitespace.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run hle`
- `README.md` — this file
