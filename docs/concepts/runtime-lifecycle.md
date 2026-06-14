# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

When an evaluation runs, four things happen in order. Every mode
(container, compose, job) runs the same sequence — the mode only changes
who orchestrates it.

## The sequence

```
/entrypoint.sh            benchmark setup (Docker ENTRYPOINT)
  └─ exec "$@"            hands off to CMD
       │
       ▼
/usr/local/bin/run        framework launcher (CMD in the combination image)
  └─ process-compose      orchestrates the three steps below
       │
       ├─ /run.sh         agent solves the task
       │
       ├─ /grade.sh       verifier grades the agent's output
       │
       └─ write-result    writes /output/{task,agent,model}/result.json
```

## Step by step

### 1. `/entrypoint.sh` — benchmark setup

The benchmark's `ENTRYPOINT`. Runs as root before anything else.

- Calls `/eval-materialize-task` to unpack the current task from
  `/tasks/all.jsonl` into `/tasks/$EVAL_TASK_ID/`.
- Sets `TASK` (the prompt the agent sees) and `EXPECTED_ANSWER`.
- Ends with `exec "$@"` — this is what connects ENTRYPOINT to CMD.

Every benchmark Dockerfile sets:
```dockerfile
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
```

The combination layer (which stitches benchmark + agent + gateway into one
image) overrides CMD to `/usr/local/bin/run` — the framework launcher that
starts process-compose. The benchmark's `CMD ["/grade.sh"]` only fires if
you run the bare benchmark image without the combination layer.

### 1b. `/usr/local/bin/run` — framework launcher

Invoked by `exec "$@"` as the combination image's CMD. Prepares the
environment (output dirs, agent user, mode detection) then execs
process-compose, which runs the agent, verifier, and result writer in
dependency order. See `core/process-compose/run` for the full script.

In **single-image mode** (no external gateway), process-compose runs all
five processes: otelcol → gateway → agent → verifier → result. In
**compose/k8s mode** (ANTHROPIC_BASE_URL already set), it runs only the
last three — otelcol and gateway are sibling containers.

### 2. `/run.sh` — agent

The agent's entrypoint. Runs as unprivileged user `agent` (uid 1002).
The agent sees only:

- `TASK` — the problem to solve
- `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` — model endpoints
- `TASK_ID`, `MODEL`, `TIMEOUT`

It cannot read `/grade.sh`, `/entrypoint.sh`, task data, or gateway
config (all root-owned, mode 0700).

Every agent Dockerfile must place its script at `/run.sh`:
```dockerfile
RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
exec my-agent "$TASK"
E
ENTRYPOINT ["/run.sh"]
```

### 3. `/grade.sh` — verifier

Runs after the agent finishes. Reads the agent's output and the expected
answer, writes an integer (0 or 1) or fraction to
`/logs/verifier/reward.txt`.

Most benchmarks copy a shared grader:
```dockerfile
COPY --from=test-exact-match /test.sh /grade.sh
```

### 4. `write-result` — output

The `write-result` script reads `/logs/verifier/reward.txt` and writes
three files:

- `/output/task/result.json` — `task_id`, `benchmark`, `reward`, `passed`
- `/output/agent/result.json` — `agent`, `started_at`, `ended_at`
- `/output/model/result.json` — `model`

When write-result finishes, process-compose exits (via `exit_on_end: true`),
which exits the container.

## Key paths

| Role | Path | Set by |
|------|------|--------|
| Benchmark setup | `/entrypoint.sh` | Benchmark Dockerfile (ENTRYPOINT) |
| Framework launcher | `/usr/local/bin/run` | Combination layer (CMD) |
| Agent entrypoint | `/run.sh` | Agent Dockerfile |
| Grading script | `/grade.sh` | Benchmark Dockerfile |
| Result writer | `/usr/local/bin/write-result` | Framework (core/process-compose) |

Benchmark and agent authors only need to care about `/entrypoint.sh`,
`/run.sh`, and `/grade.sh`. The framework launcher, process-compose,
and result writing are provided by the combination layer automatically.

## Where to go next

- [Triple-mode](triple-mode.md) — the three runtimes that run this chain
- [Overview](overview.md) — what Eval Containers is
