# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

An evaluation image is built by stitching a benchmark image, an agent
image, a gateway, and runtime tooling into a single container
(`combination.Dockerfile`). What each image must provide at that
boundary is the contract. Everything else — how the pipeline runs, how
processes are orchestrated — is framework plumbing that the combination
layer adds.

## The contract

The combination Dockerfile copies specific paths from each source image.
If your image is missing one of these, the build or the run breaks.

**From the benchmark image:**

| Path | Role |
|------|------|
| `/entrypoint.sh` | ENTRYPOINT — runs before anything else; must set `TASK` |
| `/grade.sh` | Scores the agent's output; writes reward to `/logs/verifier/reward.txt` |

**From the agent image:**

| Path | Role |
|------|------|
| `/run.sh` | Agent launch script — the code that solves the task |
| `/opt/agent/` | Agent installation directory (with `install.sh`) |

That's the interface. The combination layer inherits the benchmark's
`ENTRYPOINT ["/entrypoint.sh"]` and overrides its `CMD` from `/grade.sh`
to `/usr/local/bin/run` — the framework launcher.

## What happens at runtime

Once the stitched image starts, the sequence is:

1. **Setup** — `/entrypoint.sh` runs, sets `TASK` (the prompt the agent
   will see), then `exec "$@"` hands off to the framework launcher.
2. **Agent** — `/run.sh` runs as an unprivileged `agent` user with a
   minimal environment: `TASK`, model endpoints (`OPENAI_BASE_URL`,
   `ANTHROPIC_BASE_URL`, `GOOGLE_GEMINI_BASE_URL`), `MODEL`, `TIMEOUT`.
3. **Grade** — `/grade.sh` scores the agent's output and writes a number
   (0, 1, or a fraction) to `/logs/verifier/reward.txt`.
4. **Result** — `/usr/local/bin/write-result` reads the reward and writes
   structured output to `/output/`.

How these four steps are orchestrated depends on the runtime mode.

## How each mode runs the sequence

### Single-image (container mode)

The framework launcher (`/usr/local/bin/run`) starts **process-compose**,
an in-container orchestrator that runs five processes in dependency
order: otelcol → gateway → agent (`/run.sh`) → verifier (`/grade.sh`) →
result (`write-result`).

### Compose mode

Three containers: `otelcol`, `gateway`, `runner`. The runner still uses
`/entrypoint.sh` → `/usr/local/bin/run` → process-compose, but with an
overlay that disables the in-container otelcol and gateway (they have
their own containers). Only agent → verifier → result run inside
process-compose.

### Kubernetes (Helm Job)

The Helm chart overrides the image command entirely:

```yaml
command: ["/bin/bash", "-c"]
args: ["/entrypoint.sh /usr/local/bin/run; rc=$?; /usr/local/bin/reap-sidecars; exit $rc"]
```

otelcol and gateway run as native Kubernetes sidecars. The runner goes
through the same `/entrypoint.sh` → `/usr/local/bin/run` →
process-compose chain. After the pipeline exits, `reap-sidecars` tears
down the sidecars.

## Isolation

The agent cannot see the answers. Benchmarks protect their test data
(`/tests/`, root-owned, mode 0700) and the combination layer protects
gateway config (`/opt/gateway/`). The agent process runs via `env -i`
with an explicit allow-list of variables — it never sees `TASK_ID`,
`EXPECTED_ANSWER`, or anything outside its sandbox.

## Benchmarks that override the flow

The standard flow (entrypoint → framework launcher → process-compose) is
the default, not a requirement. A benchmark with bespoke topology can
replace it.

**tau-bench** is the main example: in compose mode it replaces the runner
entrypoint with `python3 /app/agent.py` and adds a separate harness
container. In k8s it overrides `runnerArgs` in its Helm preset. Neither
path uses process-compose — but the four steps (setup → agent → grade →
result) still happen.

## Where to go next

- [Triple-mode](triple-mode.md) — more on the three runtimes
- [Overview](overview.md) — what Eval Containers is
