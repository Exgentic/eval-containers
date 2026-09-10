# bfcl

**Status:** Released — model-only native-harness benchmark; a live-run replay fixture (`bfcl-0-claude-code`, azure/FW-GLM-5.2) is recorded and replay-tested (see AUDIT.md).

Berkeley Function Calling Leaderboard — native function-calling, AST-checked.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1150 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/ShishirPatil/gorilla](https://github.com/ShishirPatil/gorilla) |
| Paper | [paper](https://arxiv.org/abs/2406.06840) |
| Dataset revision | pinned by `BENCHMARK_VERSION` (harness + data + checker from one commit) |

## What this is

BFCL grades a function call by **AST-checking the model's tool call against a
list of acceptable argument values** — not by string-matching printed text.
This image runs BFCL's own harness and AST checker for one task, in native
function-calling mode, with the model served through the fleet gateway.

It is a **model-only** benchmark (like `automationbench`): the harness is the
agent scaffold. There is no separate agent phase, no user simulator, and no
bridge — one model turn produces `tool_calls`, which the checker grades.

Scope is the six **non-live, single-turn AST categories**: `simple_python`,
`simple_java`, `simple_javascript`, `multiple`, `parallel`,
`parallel_multiple` (1150 tasks). Live, multi-turn, memory, and web-search
categories are excluded — they require dataset/network state this image does
not carry.

## What the model sees

The runner resolves `EVAL_TASK_ID` → `(category, bfcl_id)` from the build-time
map `/tasks/all.jsonl` (root-only, §5), then hands BFCL that single task. BFCL
builds the function-calling request from the task's question and function
definitions and sends it to the gateway. The task id is never exposed to a
scrubbed agent — the runner holds it.

## How it's graded

`run_bfcl.py` drives BFCL in-process:

1. It registers a runtime `ModelConfig` for the gateway handle so BFCL's
   OpenAI-compatible handler (function-calling mode) points at
   `OPENAI_BASE_URL`. BFCL refuses any unregistered `--model`; the gateway
   handle is injected rather than editing the installed package.
2. It writes `test_case_ids_to_generate.json` for the one task and runs
   generation in `--run-ids` mode (one model turn → `tool_calls`).
3. It runs BFCL's evaluation in partial-eval mode: the **AST checker** compares
   the model's call against the category's possible-value ground truth and
   writes a per-category score file.

The reward is that category's `accuracy` — for a single task, `0.0` or `1.0`
(the checker's own verdict). Any failure is fail-closed to `0`.

## Files

- `Dockerfile` — installs BFCL (pinned), builds the task index, bakes the runner
- `run_bfcl.py` — single-task in-process runner (generation + AST evaluation)
- `compose.yaml` — compose file for `eval-containers run bfcl` (model-only runner)
- `README.md` — this file
