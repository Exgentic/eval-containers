# Eval Containers

AI agent evaluations in containers. 100 benchmarks, 20 agents — ready to deploy at massive scale on any cloud.

Our goal is to deliver agent evaluations you can trust: fast to run, thin to ship, reliable in any environment, and faithful to what each benchmark really measures.

## Why Eval Containers

|                     | Cloud-native | Framework-free | Full interchangeability (agent × model × benchmark) | Speed audit | Size audit | Reliability audit | Native model tracing |
| ------------------- | :----------: | :------------: | :-------------------------------------------------: | :---------: | :--------: | :---------------: | :------------------: |
| Harbor              |       ✗      |        ✗       |                          ✗                          |      ✗      |      ✗     |         ✗         |           ✗          |
| Inspect AI          |       ✗      |        ✗       |                          ✗                          |      ✗      |      ✗     |         ✗         |           ✗          |
| **Eval Containers** |       ✓      |        ✓       |                          ✓                          |      ✓      |      ✓     |         ✓         |           ✓          |

## Quick start

> **Pre-release.** The `oci://quay.io/eval-containers/…` registry below is the
> published-future shape — the artifacts aren't public yet. For now, clone the
> repo and add `--local` to the CLI (see [Local development](#local-development))
> or use `docker compose -f benchmarks/<name>/compose.yaml up` directly.

```bash
# Set your API key
echo "OPENAI_API_KEY=sk-..." > .env

# Run one task — pure docker, no clone, no CLI  (once registry is published)
EVAL_BENCHMARK=aime EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f oci://quay.io/eval-containers/evaluate up --abort-on-container-exit

# Results
cat output/aime/0/task/result.json
```

One URL for every evaluation. Benchmark, agent, model, and task are all `EVAL_*` env vars.

Requires Docker Compose ≥ 2.34 for `oci://` support. See [offline / older Docker](#offline--older-docker) below for alternatives.

## Or use the `eval-containers` CLI

Same thing, fewer keystrokes:

```bash
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4
```

Every `EVAL_*` env var has a matching `--kebab-case` flag. Pick whichever you prefer.

## Deployment modes

Same evaluation, three runtimes. Pick whichever matches your environment.

```bash
eval-containers run aime --task-id 0 --agent codex --mode compose      # default
eval-containers run aime --task-id 0 --agent codex --mode container
eval-containers run aime --task-id 0 --agent codex --mode job
```

| Mode | Wraps | Use it for |
|---|---|---|
| `compose` *(default)* | `docker compose -f benchmarks/<x>/compose.yaml up` | Local laptop, full stack with gateway + OTel sidecars, fastest iteration. |
| `container` | `docker run -e EVAL_MODEL=... <eval-image>` | CI smoke tests, one-shot runs against an existing model proxy, minimal footprint. |
| `job` | `kubectl apply -k benchmarks/<x>/` | Kubernetes clusters. Production-scale regressions (1000s of tasks in parallel). |

### Kubernetes (`--mode job`)

Each benchmark ships a [Kustomize](https://kustomize.io/) base — apply directly with `kubectl`, no CLI needed:

```bash
kubectl apply -k benchmarks/aime/                  # canonical pairing (claude-code, task 0)
```

For non-canonical agent/task, the CLI synthesizes an overlay:

```bash
eval-containers run aime --agent codex --task-id 42 --mode job
# expands to: kubectl kustomize --load-restrictor=LoadRestrictionsNone /tmp/eval-job-overlay-… \
#           | kubectl apply -f -
```

Production users compose their own overlays on top (corp registry rewrites, NodeAffinity, NetworkPolicies, sidecar swaps, ...) by referencing the benchmark as a Kustomize resource — see the [Kustomize docs](https://kubectl.docs.kubernetes.io/guides/config_management/components/) for the composition primitives.

The CLI can pull such an overlay in for you with `--overlay <dir>` — it's added under `components:` on the synthesized Job, so the eval-axis patches (agent/model/task) and your platform patches merge. A ready-to-adapt OpenShift overlay (sets the `anyuid` service account) ships under [`examples/deployments/openshift`](examples/deployments/openshift) — see its README for the full build-and-run-on-OpenShift walkthrough:

```bash
eval-containers run aime --agent codex --mode job \
  --overlay examples/deployments/openshift \
  --registry image-registry.openshift-image-registry.svc:5000/<namespace>
```

The overlay is a directory whose `kustomization.yaml` is `kind: Component` — data you own; the CLI never encodes platform specifics itself.

The cluster needs an `eval-secrets` Secret with `OPENAI_API_KEY` and `OPENAI_API_BASE` keys.

### Building in a cluster (`--builder`)

No local Docker? Build the images inside the cluster with buildx's Kubernetes driver — same bake graph, no extra tooling. Create the builder once (after `oc login`, or with `kubectl` pointed at the cluster):

```bash
docker buildx create --driver kubernetes --name oc --use
```

Then pass `--builder` to any build — it builds in-cluster and pushes to the registry (`--builder` implies `--push`, since a remote builder can't load into local Docker):

```bash
eval-containers build eval aime --agent codex --builder oc
```

`--dry-run` on any build prints the exact `docker buildx bake …` command without running it; if the builder doesn't exist, the CLI fails with the one-line `docker buildx create` to run.

## Environment variables

All Eval Containers env vars are prefixed `EVAL_` to avoid collision with CI systems, orchestrators, and user scripts.

**Axis selection**

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK` | Which benchmark to run | — |
| `EVAL_AGENT` | Which agent to run | — |
| `EVAL_MODEL` | Which model to route calls to | — |
| `EVAL_TASK_ID` | Which task within the benchmark | `0` |

**Container versions** (which image tag to pull)

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK_TAG` | Benchmark container version | `latest` |
| `EVAL_AGENT_TAG` | Agent container version | `latest` |
| `EVAL_MODEL_TAG` | Model container version | `latest` |

**Internal software versions** (what runs inside the container)

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_BENCHMARK_VERSION` | Dataset revision inside the benchmark | built-in pin |
| `EVAL_AGENT_VERSION` | Upstream CLI version inside the agent | built-in pin |
| `EVAL_LITELLM_VERSION` | LiteLLM version inside the model | built-in pin |

**Runtime**

| Variable | Meaning | Default |
|---|---|---|
| `EVAL_TIMEOUT` | Agent timeout in seconds | `300` |
| `EVAL_REGISTRY` | Registry to pull from | `quay.io/eval-containers` |

Container tags are Docker-native (different tag → different pull). Internal versions are runtime overrides (the entrypoint installs the requested version at container start).

Every image ships with a **reproducible default**, so casual users never touch the version vars. Power users pin.

## Concepts

- **Benchmark** — a collection of tasks (AIME has 90, SWE-bench has 500)
- **Task** — a single problem within a benchmark
- **Agent** — the AI system attempting the task (Claude Code, Codex, OpenHands, SWE-agent, Plandex, ...)
- **Model** — the LLM the agent calls, routed through a logging proxy. Works with any [LiteLLM-supported provider](https://docs.litellm.ai/docs/providers) (OpenAI, Anthropic, Google, Azure, Ollama, and 100+ more).
- **Evaluation** — one benchmark + one agent + one model, defined by one Compose artifact.

## Offline / older Docker

If you're on Docker < 2.34, airgapped, or just prefer a local file:

```bash
# Fetch + flatten the compose file once (needs a machine with network)
EVAL_BENCHMARK=aime EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f oci://quay.io/eval-containers/evaluate config > aime.compose.yaml

# Transport aime.compose.yaml anywhere. Run offline:
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f aime.compose.yaml up --abort-on-container-exit
```

Or for fully airgapped deployments, bundle the images too:

```bash
docker save quay.io/eval-containers/evals/aime--codex:latest \
            quay.io/eval-containers/models/gpt-5.4:latest \
  | gzip > aime-bundle.tar.gz
```

## Local development

If you have the repo cloned and want to iterate on a benchmark or agent without pushing to the registry:

```bash
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4 --local
```

`--local` points at `benchmarks/<name>/compose.yaml` on disk instead of `oci://...`.

## Rules

All work is governed by RULES documents. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full index.

| Rules | Scope |
|-------|-------|
| [RULES.md](RULES.md) | Core principles |
| [benchmarks/RULES.md](benchmarks/RULES.md) | Building benchmarks |
| [agents/RULES.md](agents/RULES.md) | Building agents |
| [models/RULES.md](models/RULES.md) | Building models |
| [src/RULES.md](src/RULES.md) | CLI |
| [compose/RULES.md](compose/RULES.md) | Naming, compose, output, registry |

## Setup

- [tests/LOCAL.md](tests/LOCAL.md) — local dev loop (Docker Desktop, Podman, Rosetta)
- [RELEASE.md](RELEASE.md) — how CI builds and publishes the fleet
