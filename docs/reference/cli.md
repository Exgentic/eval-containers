# CLI reference

*Reference · for operators & contributors · derives from `src/` (`main.rs`, `run.rs`, `build.rs`) and [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md). The source is authoritative; run `eval-containers --help` for the exact, current flags.*

The `eval-containers` CLI is optional — every command maps to a plain
`docker` / `helm` / `kubectl` / `oc` command you could type yourself. State- or
outward-changing commands support `--dry-run` to print that command without
running it.

## Global

```
eval-containers [--registry <ref>] <command> [args]
```

| Flag | Env | Default |
|---|---|---|
| `--registry <ref>` | `EVAL_REGISTRY` | `quay.io/eval-containers` |

## Commands

| Command | Does | Wraps |
|---|---|---|
| `run` | Run an evaluation | `docker compose` / `docker run` / `helm template \| kubectl apply` |
| `build` | Build images (agents, benchmarks, models, eval combos) | `docker buildx bake` / `docker build` |
| `push` | Push images to the registry | `docker push` |
| `list` | List images with metadata | reads the repo |
| `images` | Show images with sizes | `docker images` |
| `inspect` | Inspect an image | `docker inspect` |
| `prune` | Reclaim disk | `docker builder prune` + `docker image prune` |
| `report` | Aggregate results: pass/reward/tokens/cost + traces health | reads `output/` |
| `gen-bake` | Scaffold a `docker-bake.hcl` for an artifact | writes a file |
| `oracle` | Validate a benchmark's grading: a gold solution must score 1.0 and a no-op < 1.0 through the benchmark's own grader (no agent, no model). See [Oracle](../../core/oracle/README.md). | `docker run` against the grader |

## `run` flags

`eval-containers run [BENCHMARK] [flags]` — `BENCHMARK` is a positional
shortcut for `--benchmark`. Every `EVAL_*` axis has a matching flag; the flag
overrides the env var.

| Flag | Maps to | Notes |
|---|---|---|
| `--benchmark <name>` | `EVAL_BENCHMARK` | or positional |
| `--agent <name>` | `EVAL_AGENT` | |
| `--model <name>` | `EVAL_MODEL` | sets the gateway upstream |
| `--task-id <id>` | `EVAL_TASK_ID` | default `0` |
| `--mode <compose\|container\|job>` | — | default `compose` |
| `--benchmark-tag <tag>` | `EVAL_BENCHMARK_TAG` | image tag |
| `--agent-tag <tag>` | `EVAL_AGENT_TAG` | image tag |
| `--model-tag <tag>` | `EVAL_MODEL_TAG` | image tag |
| `--benchmark-version <v>` | `EVAL_BENCHMARK_VERSION` | dataset revision inside the image |
| `--agent-version <v>` | `EVAL_AGENT_VERSION` | upstream CLI version inside the image |
| `--litellm-version <v>` | `EVAL_LITELLM_VERSION` | LiteLLM version inside the image |
| `--timeout <secs>` | `EVAL_TIMEOUT` | default `300` |
| `--max-budget <usd>` | `EVAL_MODEL_MAX_BUDGET` | hard spend cap; default `$1` |
| `--local` | — | use in-repo `benchmarks/<name>/` instead of the registry |
| `--dry-run` | — | print/validate without deploying (`job`: `kubectl --dry-run=server`) |
| `-n, --namespace <ns>` | — | `job` mode only; `kubectl -n` |
| `--overlay <values.yaml>` | — | `job` mode only; extra `helm -f` (e.g. `deploy/values-openshift.yaml`) |

See [Environment variables](env-vars.md) for the full `EVAL_*` namespace.

## `build` flags

`eval-containers build <agent|bench|model|eval> <name> [flags]`

`eval-containers build compose` takes no name — it publishes the generic
compose artifact to `oci://<registry>/evaluate`, parameterized at run time by
`EVAL_BENCHMARK` / `EVAL_AGENT`.

| Flag | Notes |
|---|---|
| `--builder <name>` | build with a named buildx builder (e.g. in-cluster `--driver kubernetes`); **implies `--push`** |
| `--dry-run` | print the `docker buildx bake` command without running it |

If the named builder doesn't exist, the command fails with the exact
`docker buildx create` line to run.
