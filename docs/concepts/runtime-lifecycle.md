# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`doctrine/benchmarks/RULES.md`](../../doctrine/benchmarks/RULES.md) rule 12.*

When an evaluation runs, four things happen in order. Every mode
(container, compose, job) runs the same sequence — the mode only changes
who orchestrates it.

## The sequence

```
/entrypoint.sh       benchmark setup (Docker ENTRYPOINT)
  └─ exec "$@"       hands off to the default command
       │
       ▼
/run.sh              agent solves the task
       │
       ▼
/grade.sh            verifier grades the agent's output
       │
       ▼
result.json          final reward written to /output/task/
```

## Step by step

### 1. `/entrypoint.sh` — benchmark setup

The benchmark's `ENTRYPOINT`. Runs as root before anything else.

- Calls `/eval-materialize-task` to unpack the current task from
  `/tasks/all.jsonl` into `/tasks/$EVAL_TASK_ID/`.
- Sets `TASK` (the prompt the agent sees) and `EXPECTED_ANSWER`.
- Ends with `exec "$@"` — this is what connects ENTRYPOINT to CMD.

Every benchmark Dockerfile must set:
```dockerfile
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
```

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

### 4. `result.json` — output

The framework reads `/logs/verifier/reward.txt` and writes the final
`/output/task/result.json` with `task_id`, `benchmark`, `reward`, and
`passed`. When this completes, the container exits.

## The two path conventions

| Role | Path | Set by |
|------|------|--------|
| Agent entrypoint | `/run.sh` | Agent Dockerfile |
| Grading script | `/grade.sh` | Benchmark Dockerfile |

These are the only paths a benchmark or agent author needs to know.
Everything else (`/entrypoint.sh`, the framework launcher, result
writing) is provided by the framework and inherited automatically.

## Where to go next

- [Triple-mode](triple-mode.md) — the three runtimes that run this chain
- [Overview](overview.md) — what Eval Containers is
