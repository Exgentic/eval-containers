# SkillsBench

94 expert tasks across 11 domains (software engineering, mathematics, cybersecurity,
office/white-collar, industrial systems, and more). Evaluates how well agents handle
realistic, high-value tasks that require domain expertise.

- **Paper:** https://arxiv.org/abs/2602.12670
- **Website / Leaderboard:** https://www.skillsbench.ai/
- **Upstream repo:** https://github.com/benchflow-ai/skillsbench

## Task model

Per-task images: `EVAL_TASK_ID` is a build ARG set to the task name (e.g. `citation-check`).
Each image bakes one task's input files, instruction, and test suite.

Currently supported tasks: `citation-check`

## Agent contract

- Input: `$TASK` env var — the full task instruction from `instruction.md`
- Working directory: `/root` — task input files are present here at start
- Output: agent writes results to `/root/` per the instruction (e.g. `/root/answer.json`)
- Scoring: custom pytest verifier per task; reward is `1.0` (pass) or `0.0` (fail)

## Build

```bash
# Build the benchmark base image for citation-check
docker build --build-arg EVAL_TASK_ID=citation-check \
  -t local/benchmark-skills-bench:citation-check benchmarks/skills-bench/

# Build the eval combination (benchmark + agent + model)
eval-containers build eval skills-bench-citation-check --agent claude-code
```

## Run

```bash
# Compose mode (local dev)
EVAL_TASK_ID=citation-check EVAL_AGENT=claude-code EVAL_MODEL=claude-sonnet-4-6 \
  docker compose -f benchmarks/skills-bench/compose.yaml up --abort-on-container-exit

# Check result
cat output/skills-bench/citation-check/task/result.json
```

## Leaderboard reference (without skills)

| Agent + Model | Pass rate |
|---|---|
| OpenHands / GPT-5.5 | 37.9% |
| Claude Code / Opus 4.7 | 29.8% |
| OpenHands / Sonnet 4.6 | 19.7% |

## Adding a new task

1. Check the task's `environment/Dockerfile` in the upstream repo for its apt and pip deps
2. Add those deps to the `Dockerfile` in this directory (in the per-task deps section)
3. Update `eval.benchmark.tasks` label comment and this README
4. Build and test: `docker build --build-arg EVAL_TASK_ID=<new-task> ...`
