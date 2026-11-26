# longbench-v2

LongBench v2 - long-context multiple-choice reasoning (8k-2M words), official
0-shot prompt and answer extraction. 503 questions across six task categories
(single- and multi-document QA, long in-context learning, long dialogue history,
code repositories, long structured data), each with four choices (A/B/C/D).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 503 (ids `0`–`502`, dataset order; upstream `_id` in `id.txt`) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [THUDM/LongBench](https://github.com/THUDM/LongBench) (LongBench v2 code at the repo root) |
| Dataset | [zai-org/LongBench-v2](https://huggingface.co/datasets/zai-org/LongBench-v2) (formerly `THUDM/LongBench-v2`) |
| Paper | [arXiv:2412.15204](https://arxiv.org/abs/2412.15204) |
| Dataset revision | `2b48e494f2c7a2f0af81aae178e05c7e1dde0fe9` |
| Prompt / grader source | `prompts/0shot.txt`, `pred.py` at THUDM/LongBench `2e00731f8d0bff23dc4325161044d0ed8af94c1e` |

## Protocol

Follows upstream's `pred.py` in its default (w/o-CoT) setting:

- **Prompt.** `prompts/0shot.txt` verbatim, filled with the stripped `context`,
  `question` and `choice_A`–`choice_D`, in upstream's substitution order. It
  ends: `Format your response as follows: "The correct answer is (insert answer here)".`
- **Answer extraction.** Upstream's `extract_answer`: remove every `*`, take the
  first `The correct answer is \(([A-D])\)`, else the first
  `The correct answer is ([A-D])`. No match is wrong.
- **Score.** `1` iff the extracted letter equals the gold `answer`, else `0`.
  The suite accuracy is the mean over the 503 tasks (upstream `result.py`
  "Overall"; random is 25%).

Why 0-shot rather than CoT: upstream's `--cot` mode is a two-call scaffold for
non-reasoning chat models (`0shot_cot.txt`, then `0shot_cot_ans.txt` with the
first reply pasted in). The [leaderboard](https://longbench2.github.io/#leaderboard)
is sorted by the w/ CoT column, and lists reasoning models (o1, DeepSeek-R1 —
"natively use CoT") only there. An agent here answers in a single turn and
reasons on its own, which is the reasoning-model setting; the one-turn prompt it
gets is `0shot.txt`. Compare a reasoning model's score here with the w/ CoT
column.

### Where this differs from the published setting

- **Context access.** Upstream puts the whole prompt in one chat message,
  middle-truncated to the model's window (`config/model2maxlen.json`: 120k
  tokens for most models — keep the first and last 60k). Here the rendered prompt
  (48 KB to 16 MB) is too large for the `TASK` env var, so, like `longbench` and
  `mrcr`, it is written to `/app/task.txt` and the agent reads it with its own
  tools; nothing is truncated. Scores therefore measure the model *through its
  agent*: an agent that reads the file in full approximates the published
  setting, one that searches it does not. A model-only native harness (the
  `cvdp` pattern) would be the like-for-like comparison.
- **Decoding.** Upstream samples at `temperature=0.1`, `max_tokens=128`; here
  the agent's own decoding settings apply.

## What the agent sees

`TASK`: "Your task is in /app/task.txt: a long text followed by a multiple-choice
question at the end of the file. Read it carefully and answer exactly in the
format it asks for." `/app/task.txt` is the rendered `0shot.txt` prompt. The gold
letter and the task store (`/tasks`, root-only) are not readable by the agent.

## How it's graded

`grade.sh` runs `grade.py` (root-only at `/tests/grade.py`) — upstream's
`extract_answer` on the agent's stdout (`/output/agent/stdout.log`) and writes `1` or `0` to
`/logs/verifier/reward.txt`; it fails closed to `0`.

## Oracle

`solution.sh` prints `The correct answer is (<gold>)`:

```bash
eval-containers oracle longbench-v2 --local   # gold=1, no-op=0
```

## Eval image build args

`core/combination.Dockerfile` with `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/longbench-v2:latest`,
`AGENT_IMAGE=ghcr.io/exgentic/agents/<agent>:latest`, `AGENT_VERSION=<agent version>`.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run longbench-v2`
- `docker-bake.hcl` — bake target `benchmark-longbench-v2`
- `materialize.py` — build-time loader: renders `0shot.txt` per task into `/tasks/all.jsonl`
- `entrypoint.sh`, `grade.sh`, `grade.py` — task setup and the grader
- `solution.sh` — oracle gold solution (mounted at oracle time, never baked)
- `README.md` — this file
