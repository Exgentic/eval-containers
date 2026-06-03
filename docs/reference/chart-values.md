# Chart values reference

*Reference · for operators · derives from [`benchmarks/_chart/values.yaml`](../../benchmarks/_chart/values.yaml). That file is authoritative — these are its fields with defaults at the time of writing.*

The shared chart `benchmarks/_chart` renders the otelcol + gateway + runner Job.
A benchmark's `values.yaml` sets `benchmark` (required) and overrides only what
differs; per-run axes arrive via `--set` (or the CLI). See
[The Helm chart](../concepts/the-helm-chart.md).

## Required

| Field | Meaning |
|---|---|
| `benchmark` | Benchmark name. The only field most `values.yaml` files set. |

## Per-run axes — *set at deploy via `--set` / the CLI*

| Field | Default | CLI flag |
|---|---|---|
| `agent` | `claude-code` | `--agent` |
| `task` | `"0"` | `--task-id` |
| `registry` | `quay.io/eval-containers` | `--registry` |
| `evalModel` | `openai/azure/gpt-5.4` | `--model` (also sets `model`) |
| `model` | `gpt-5.4-bifrost` | runner's logging tag |
| `gatewayImage` | `gpt-5.4--bifrost` | the proxy image |
| `gatewayTag` | `latest` | `--model-tag` |
| `runnerTag` | `latest` | `--agent-tag` / `--benchmark-tag` |
| `benchmarkVersion` | `""` | `--benchmark-version` |
| `agentVersion` | `""` | `--agent-version` |
| `litellmVersion` | `""` | `--litellm-version` |
| `maxBudget` | `""` | `--max-budget` |

## Knobs a benchmark may override

| Field | Default |
|---|---|
| `timeout` | `"300"` |
| `activeDeadlineSeconds` | `900` |
| `runnerArgs` | `/entrypoint.sh; rc=$?; /usr/local/bin/reap-sidecars; exit $rc` |
| `resources.requests` | `{ cpu: 500m, memory: 512Mi }` |
| `resources.limits` | `{ cpu: 2, memory: 2Gi }` |

## Platform composition — *layer via `--overlay` (extra `-f`)*

| Field | Default | Notes |
|---|---|---|
| `serviceAccountName` | `""` | OpenShift sets `anyuid-sa` |
| `sweepId` | `""` | sweep bookkeeping |

## Composition hooks — *for benchmarks with bespoke topology*

Standard benchmarks leave these empty; the bespoke few (osworld, tau-bench,
visualwebarena, webarena) use them to add sidecars, Deployments, and Services.

| Field | Default | Example use |
|---|---|---|
| `initContainers` | `[]` | a wait-for-`<service>` gate |
| `runnerExtraEnv` | `[]` | `DESKTOP_URL` for osworld |
| `gatewayExtraEnv` | `[]` | |
| `runnerExtraVolumeMounts` | `[]` | |
| `extraVolumes` | `[]` | |
| `extraManifests` | `[]` | extra Deployments/Services (full manifests) |
