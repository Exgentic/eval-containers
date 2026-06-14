# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

When you launch an evaluation, four things happen in sequence: the task
is prepared, the agent works on it, a grader scores the work, and the
result is written out. This page walks you through that sequence — first
the common path, then how each runtime mode implements it, then the
places where benchmarks diverge.

## What happens when an eval runs

### 1. Setup — "what should the agent do?"

The benchmark's entrypoint (`/entrypoint.sh`) prepares the task. In the
common case it calls `/eval-materialize-task`, which reads the task list
at `/tasks/all.jsonl`, unpacks the row matching `EVAL_TASK_ID` into
`/tasks/$EVAL_TASK_ID/`, and exports a `TASK` environment variable —
the plain-text prompt the agent will see.

Some benchmarks also set grader-specific variables here (e.g.
`EXPECTED_ANSWER` for exact-match graders). These are conventions of
individual graders, not part of the contract.

Not every benchmark works this way. Per-task benchmarks like swe-bench
and terminal-bench bake one task per image at build time — their
entrypoint sets `TASK` directly and never calls `/eval-materialize-task`.
The only requirement is that `TASK` is set by the time the agent starts.

### 2. Agent — "solve this"

The agent runs as an unprivileged `agent` user. (The framework creates
this user with uid 1002 as a fallback if the image didn't already have
one.) It gets a minimal environment:

- `TASK` — the problem to solve
- `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL`
  — model endpoints, always pointing at the gateway proxy, never at a
  provider directly
- `MODEL` — which model to request
- `TIMEOUT` — wall-clock limit

The agent cannot see the answers. Benchmarks protect their test data
(`/tests/`, root-owned, mode 0700) and the combination layer protects
gateway config (`/opt/gateway/`). The scripts `/grade.sh` and
`/entrypoint.sh` are world-executable but reading them is usually
harmless — the secrets are in the data, not the scripts.

The agent's code lives at `/run.sh`, provided by the agent's Dockerfile.

### 3. Grade — "how did it do?"

The grader scores the agent's output and writes a number to
`/logs/verifier/reward.txt` — an integer (0 or 1) or a fraction. How it
scores is benchmark-specific: compare against `EXPECTED_ANSWER`, call a
judge LLM, run a test suite, or something entirely custom.

The grading script lives at `/grade.sh`. Most benchmarks copy a shared
grader image:
```dockerfile
COPY --from=test-exact-match /test.sh /grade.sh
```

### 4. Result — "write it down"

`/usr/local/bin/write-result` reads `/logs/verifier/reward.txt` and
writes three structured files:

- `/output/task/result.json` — `task_id`, `benchmark`, `reward`, `passed`
- `/output/agent/result.json` — `agent`, `started_at`, `ended_at`
- `/output/model/result.json` — `model`

This is what the outside world reads to know what happened.

## How each runtime mode wires the sequence

The four steps are always the same. What changes is who starts each step
and where the processes live.

### Single-image (container mode)

Everything runs in one container. The Docker image's ENTRYPOINT and CMD
chain the whole sequence:

```
ENTRYPOINT ["/entrypoint.sh"]  →  exec "$@"  →  CMD ["/usr/local/bin/run"]
```

`/entrypoint.sh` does setup (step 1), then hands off to
`/usr/local/bin/run` — the **framework launcher**. It starts
**process-compose**, an in-container orchestrator that runs five
processes in dependency order: otelcol → gateway → agent (`/run.sh`) →
verifier (`/grade.sh`) → result (`write-result`).

### Compose mode

Three containers: `otelcol`, `gateway`, `runner`. The runner still uses
`/entrypoint.sh` → `/usr/local/bin/run` → process-compose, but with an
overlay that disables the in-container otelcol and gateway (they have
their own containers now). Only agent → verifier → result run inside
process-compose.

### Kubernetes (Helm Job)

The Helm chart overrides the image command entirely:

```yaml
command: ["/bin/bash", "-c"]
args: ["/entrypoint.sh /usr/local/bin/run; rc=$?; /usr/local/bin/reap-sidecars; exit $rc"]
```

otelcol and gateway run as native Kubernetes sidecars (init containers
with `restartPolicy: Always`). The runner goes through the same
`/entrypoint.sh` → `/usr/local/bin/run` → process-compose chain. After
the pipeline exits, `reap-sidecars` tears down the sidecars.

## Benchmarks that skip the standard flow

The standard flow (entrypoint → framework launcher → process-compose) is
the default, not a requirement. A benchmark with bespoke topology can
replace it.

**tau-bench** is the main example: in compose mode it replaces the runner
entrypoint with `python3 /app/agent.py` and adds a separate harness
container that calls `/eval-materialize-task` itself. In k8s it overrides
`runnerArgs` in its Helm preset. Neither path uses process-compose — but
the four steps (setup → agent → grade → result) still happen.

## Key paths at a glance

| What | Path | Who provides it |
|------|------|-----------------|
| Benchmark entrypoint | `/entrypoint.sh` | You (benchmark Dockerfile) |
| Task unpacker | `/eval-materialize-task` | Framework — most benchmarks use it |
| Framework launcher | `/usr/local/bin/run` | Framework (combination layer) |
| Agent code | `/run.sh` | You (agent Dockerfile) |
| Grading script | `/grade.sh` | You (benchmark Dockerfile) |
| Result writer | `/usr/local/bin/write-result` | Framework |

If you're writing a benchmark, you provide `/entrypoint.sh` and
`/grade.sh`. If you're writing an agent, you provide `/run.sh`.
Everything else comes from the framework.

## Where to go next

- [Triple-mode](triple-mode.md) — more on the three runtimes
- [Overview](overview.md) — what Eval Containers is
