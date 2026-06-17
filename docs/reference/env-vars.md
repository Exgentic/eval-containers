# Environment variables

*Reference · for operators · derives from `src/run.rs` and [`.agents/src/RULES.md`](../../.agents/src/RULES.md). This page is the authoritative list of `EVAL_*` variables.*

All Eval Containers variables are prefixed `EVAL_` to avoid collision with CI
systems, orchestrators, and user scripts. Every variable has a matching
`--kebab-case` CLI flag (see [CLI reference](cli.md)); the flag overrides the
env var.

## Axis selection

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK` | Which benchmark to run | — |
| `EVAL_AGENT` | Which agent to run | — |
| `EVAL_MODEL` | Which model to route calls to | — |
| `EVAL_TASK_ID` | Which task within the benchmark | `0` |

## Container versions — *which image tag to pull*

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK_TAG` | Benchmark container version | `latest` |
| `EVAL_AGENT_TAG` | Agent container version | `latest` |
| `EVAL_MODEL_TAG` | Model container version | `latest` |

## Internal software versions — *what runs inside the container*

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK_VERSION` | Dataset revision inside the benchmark | built-in pin |
| `EVAL_AGENT_VERSION` | Upstream CLI version inside the agent | built-in pin |
| `EVAL_LITELLM_VERSION` | LiteLLM version inside the model | built-in pin |

## Runtime

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_TIMEOUT` | Agent timeout in seconds | `300` |
| `EVAL_MODEL_MAX_BUDGET` | Hard cap on model spend (USD) for this run | `1` |
| `EVAL_REGISTRY` | Registry to pull from | `ghcr.io/exgentic` |

The two version axes are orthogonal: the **tag** controls which container to
pull (Docker-native), the **version** is a runtime override the entrypoint
installs at container start. Every image ships a reproducible default, so casual
users never set these — see [Overview → Two version axes](../concepts/overview.md).
