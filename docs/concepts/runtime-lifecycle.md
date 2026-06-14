# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

Every evaluation follows the same four-step contract, regardless of how
it is orchestrated. The mode (container, compose, k8s Job) changes who
starts each step and where the processes live — the steps themselves are
the same.

## The contract

```
  setup        materialize the task, set TASK
     │
     ▼
  agent        solve the task (sees only TASK + model endpoints)
     │
     ▼
  grade        score the agent's output → reward
     │
     ▼
  result       write structured output to /output/
```

### 1. Setup — task materialization

Set the `TASK` environment variable (the prompt the agent sees) and
prepare any data the grader will need. The benchmark may also set
grader-specific variables (e.g. `EXPECTED_ANSWER` for exact-match
benchmarks) — these are conventions of individual graders, not part of
the contract.

Most benchmarks do this by calling `/eval-materialize-task` in their
`/entrypoint.sh`, which unpacks the current task from `/tasks/all.jsonl`
into `/tasks/$EVAL_TASK_ID/`. Per-task benchmarks (swe-bench,
terminal-bench) bake task data into the image at build time and skip
`/eval-materialize-task` entirely — their entrypoint just sets `TASK`
directly.

### 2. Agent

Run as unprivileged user `agent`. (The framework creates this user with
uid 1002 as a fallback if the image didn't pre-create it.) The agent
sees only:

- `TASK` — the problem to solve
- `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL`
  — model endpoints (the gateway, never a direct provider URL)
- `MODEL`, `TIMEOUT`

It cannot read test data or gateway config — benchmarks protect `/tests/`
and the combination layer protects `/opt/gateway/` (root-owned, mode
0700). The scripts `/grade.sh` and `/entrypoint.sh` themselves are
world-executable.

Standard path: `/run.sh`, placed by the agent Dockerfile.

### 3. Grade

Score the agent's output and write an integer (0 or 1) or fraction to
`/logs/verifier/reward.txt`. How scoring works is benchmark-specific —
exact-match against `EXPECTED_ANSWER`, a judge LLM call, a test suite,
or something custom.

Standard path: `/grade.sh`, placed by the benchmark Dockerfile. Most
benchmarks copy a shared grader:
```dockerfile
COPY --from=test-exact-match /test.sh /grade.sh
```

### 4. Result

Read `/logs/verifier/reward.txt` and write structured output:

- `/output/task/result.json` — `task_id`, `benchmark`, `reward`, `passed`
- `/output/agent/result.json` — `agent`, `started_at`, `ended_at`
- `/output/model/result.json` — `model`

Standard path: `/usr/local/bin/write-result`.

## How each mode runs the contract

### Single-image (container mode)

Everything in one container. The Docker image's ENTRYPOINT and CMD wire
the whole chain:

```
ENTRYPOINT ["/entrypoint.sh"]  →  exec "$@"  →  CMD ["/usr/local/bin/run"]
```

`/usr/local/bin/run` (the framework launcher) starts **process-compose**,
which orchestrates all five processes in dependency order:
otelcol → gateway → agent (`/run.sh`) → verifier (`/grade.sh`) → result
(`write-result`).

### Compose mode

Three containers: `otelcol`, `gateway`, `runner`. The runner still uses
`/entrypoint.sh` → `/usr/local/bin/run` → process-compose, but with an
overlay (`process-compose-runner.yaml`) that disables the in-container
otelcol and gateway — only agent → verifier → result run inside
process-compose. Networking changes from `localhost` to Docker service
names.

### Kubernetes (Helm Job)

The chart overrides the image command entirely:

```yaml
command: ["/bin/bash", "-c"]
args: ["/entrypoint.sh /usr/local/bin/run; rc=$?; /usr/local/bin/reap-sidecars; exit $rc"]
```

otelcol and gateway run as native sidecars (init containers with
`restartPolicy: Always`). The runner still goes through `/entrypoint.sh`
→ `/usr/local/bin/run` → process-compose (runner-only mode), then
`reap-sidecars` tears down the sidecars after the pipeline exits.

## Benchmarks that override the flow

The standard flow (entrypoint → framework launcher → process-compose) is
the default, not a requirement. A benchmark with bespoke topology can
override it entirely.

**tau-bench** is the main example: in compose mode it replaces the runner
entrypoint with `python3 /app/agent.py` and adds a separate harness
container that calls `/eval-materialize-task` itself. In k8s it overrides
`runnerArgs` in its Helm preset. Neither path uses process-compose — but
the four-step contract (setup → agent → grade → result) still holds.

## Key paths

| Role | Path | Set by |
|------|------|--------|
| Benchmark setup | `/entrypoint.sh` | Benchmark Dockerfile (ENTRYPOINT) |
| Task unpacker (most benchmarks) | `/eval-materialize-task` | Framework (core/entrypoint) |
| Framework launcher | `/usr/local/bin/run` | Combination layer (CMD) |
| Agent entrypoint | `/run.sh` | Agent Dockerfile |
| Grading script | `/grade.sh` | Benchmark Dockerfile |
| Result writer | `/usr/local/bin/write-result` | Framework (core/process-compose) |

Benchmark and agent authors need to provide `/entrypoint.sh`, `/run.sh`,
and `/grade.sh`. Everything else is inherited from the framework.

## Where to go next

- [Triple-mode](triple-mode.md) — the three runtimes that run this chain
- [Overview](overview.md) — what Eval Containers is
