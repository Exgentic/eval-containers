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
| `EVAL_MODEL` | LiteLLM handle `<provider>/<model>` the gateway routes to (e.g. `openai/gpt-5.4`) — **required**, must be `<provider>/<model>` form | — |
| `EVAL_TASK_ID` | Which task within the benchmark | `0` |
| `EVAL_GATEWAY` | Which proxy image serves the model | `bifrost` |

Model and gateway are **independent axes** ([gateways/RULES.md](../../.agents/gateways/RULES.md)):
`EVAL_MODEL` is a *runtime handle, not an image* — any LiteLLM-supported
provider/model works with no per-model build — and `EVAL_GATEWAY` names the
proxy image that routes it (default `bifrost`; also `litellm`, `portkey`).
Set `EVAL_GATEWAY` to a **pinned per-model image** (e.g. `gpt-5.4`) — a baked,
shared artifact that ignores `EVAL_MODEL`. Both are pull-not-build; see
[Use a model](../guides/add-a-model.md).

> **Renamed.** `EVAL_GATEWAY_IMAGE` became `EVAL_GATEWAY` and `EVAL_MODEL_TAG`
> became `EVAL_GATEWAY_TAG` (both always addressed the gateway image). Old names
> are **refused by name**, never aliased: set one and the CLI stops and tells you
> the new spelling. Same for the flags — `--model-tag`, `build eval --model`, the
> deploy scripts' `--eval-model`, and a `--model` naming a proxy image. One live
> spelling per axis is the point of the rename; a *renamed artifact* gets its
> compatibility from the registry instead (the same digest published under both
> paths), so nothing in the code carries two names.

## Container versions — *which image tag to pull*

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK_TAG` | Benchmark container version | `latest` |
| `EVAL_AGENT_TAG` | Agent container version | `latest` |
| `EVAL_GATEWAY_TAG` | Gateway container version | `latest` |

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
| `EVAL_AGENT_REASONING_EFFORT` | Reasoning effort the agent applies (`low`/`medium`/`high`; some also accept `xhigh`/`max`) | agent default |
| `EVAL_REGISTRY` | Registry to pull from | `ghcr.io/exgentic` |

Supported agents: **codex, claude-code, claude-code-rtk, aider, cline,
copilot-cli, openclaw**. Setting it for any other agent **fails loud** (the run
exits non-zero) rather than silently ignoring it.

The two version axes are orthogonal: the **tag** controls which container to
pull (Docker-native), the **version** is a runtime override the entrypoint
installs at container start. Every image ships a reproducible default, so casual
users never set these — see [Overview → Two version axes](../concepts/overview.md).
