# skills-bench

SkillsBench (benchflow-ai/skillsbench) — 86 expert tasks across 11 domains
(software engineering, mathematics, cybersecurity, office/white-collar, industrial
systems, and more), built from upstream source.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 86 (`tasks/<task>` upstream) |
| Environment | per-task |
| Internet required | true (agent — skills fetch data) · true (build) |
| Released | no |
| Upstream | [github.com/benchflow-ai/skillsbench](https://github.com/benchflow-ai/skillsbench) |
| Paper | [arxiv.org/abs/2602.12670](https://arxiv.org/abs/2602.12670) |
| Leaderboard | [skillsbench.ai](https://www.skillsbench.ai/) |

## What the agent sees

Each task is an environment plus an instruction. The instruction is the task's
`instruction.md`, copied to `/task/instruction.md` and passed to the agent via the
`TASK` env var. Task input files come from the task's own `environment/Dockerfile`.

**Skills.** SkillsBench's defining feature: each task ships `environment/skills/`
(expert `SKILL.md` knowledge + scripts + references). They are copied to
`./.claude/skills/` in the agent's working directory, where claude-code
auto-discovers them as project skills in `-p` mode — no agent or launcher change
needed, because `claude` runs in the eval image's `WORKDIR` (the benchmark is the
`FROM` base) and never `cd`s. Verified end-to-end on `citation-check`: the agent
invoked the `Skill` tool for `citation-management`, ran its scripts, and solved
the task. (Skill discovery is claude-code-specific; the no-skills baseline is the
benchmark built without this copy.)

## How it's graded

`/grade.sh` runs the task's upstream `tests/test_outputs.py` with pytest
(pre-installed at build, so grading needs no network) and writes `1`/`0` to
`/logs/verifier/reward.txt`; a task that emits a continuous score (civ6) writes it
to `scores/` and that value becomes the reward. The tests live in a **root-only**
`/tests` (chmod 700) — the agent can read neither the tests nor any solution
(benchmarks/RULES.md rule 9 / eval integrity).

Upstream's `tests/test.sh` fetches pytest via `uvx` at run time; we install it at
build instead, to keep grading offline and reproducible.

## Per-task build (built from source)

No per-task upstream images exist, and each task ships its own
`environment/Dockerfile` (heterogeneous base + setup). So `build.sh` builds each
per-task image in two steps (benchmarks/RULES.md 24g):

1. build the task's **own** `environment/Dockerfile` → the task env;
2. overlay our eval pipeline (`Dockerfile`, `FROM ${TASK_BASE}`) → instruction,
   root-only tests, grader, entrypoint.

Both steps fetch the upstream task dir at the pinned `SB_REF` (in `build.sh`)
directly from the builder — no local checkout.

## Oracle

`solution.sh` fetches **this task's** upstream `solution/solve.sh` at `SB_REF` and
runs it — derived, never hardcoded, and fetched fresh at oracle run time (never
baked into the agent image). Verify:

```bash
eval-containers oracle skills-bench --task-id citation-check --local
```

## Files

- `Dockerfile` — the eval overlay (`FROM ${TASK_BASE}`); not built directly, see `build.sh`
- `build.sh` — the two-step per-task build (task env → overlay)
- `solution.sh` — oracle gold (fetches the per-task upstream `solve.sh`)
- `container.Dockerfile` — single-image pin (`evals/skills-bench-<task>--<agent>`)
- `compose.yaml` — compose file for `eval-containers run skills-bench`
- `README.md` — this file
