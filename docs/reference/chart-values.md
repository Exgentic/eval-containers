# Chart values reference

*Reference · for operators · derives from [`containers/benchmarks/_chart/values.yaml`](../../containers/benchmarks/_chart/values.yaml). That file is authoritative — these are its fields with defaults at the time of writing.*

The shared chart `containers/benchmarks/_chart` renders the runner Job, with an otelcol sidecar and an opt-in gateway sidecar.
The benchmark is named via `--set benchmark=<x>` (required); a benchmark with
bespoke topology overrides what differs through an optional `presets/<x>.yaml`
in the chart. Per-run axes arrive via `--set` (or the CLI). See
[The Helm chart](../concepts/the-helm-chart.md).

## Required

| Field | Meaning |
|---|---|
| `benchmark` | Benchmark name, set via `--set benchmark=<x>`. Selects the optional `presets/<x>.yaml` overlay. |

## Per-run axes — *set at deploy via `--set` / the CLI*

| Field | Default | CLI flag |
|---|---|---|
| `agent` | `claude-code` | `--agent` (a native-harness benchmark's preset pins `native`) |
| `task` | `"0"` | `--task-id` |
| `registry` | `ghcr.io/exgentic` | `--registry` |
| `model` | — *(required)* | `--model` — the `<provider>/<model>` handle the gateway routes (the runner's clean label is derived from it) |
| `apiKeyHeader` | `""` | `--api-key-header` — upstream credential header; empty uses wire-native auth; Bifrost requires `EVAL_MODEL_API=openai` in `gatewayExtraEnv`; see [edge configuration](../../containers/core/edge/README.md#configuration) |
| `gatewayEnabled` | `false` | set by `--gateway` / `--gateway-tag` — run the gateway sidecar; off, the edge forwards straight to the upstream |
| `offlineGateway` | `{podSelector: {}, port: ""}` | the platform overlay — the labels and port of the in-namespace gateway Service an offline benchmark's egress NetworkPolicy allows; an offline run without it, or with `gatewayEnabled`, is refused (#753) |
| `dnsNamespace` | `kube-system` | the platform overlay (`openshift-dns`) — the namespace of the cluster DNS, the only one that policy lets an offline pod reach, on the DNS ports |
| `gatewayImage` | `bifrost` | `--gateway` — the proxy backend (`bifrost` / `litellm` / `portkey`, or a pinned per-model image) |
| `gatewayTag` | `latest` | `--gateway-tag` |
| `runnerTag` | `latest` | `--agent-tag` / `--benchmark-tag` |
| `benchmarkVersion` | `""` | `--benchmark-version` |
| `agentVersion` | `""` | `--agent-version` |
| `litellmVersion` | `""` | `--litellm-version` |
| `maxBudget` | `""` | `--max-budget` |
| `maxTokens` | `""` | `--max-tokens` |
| `reasoningEffort` | `""` | `--agent-reasoning-effort` |
| `modelContextWindow` | `""` | `--model-context-window` |
| `agentEnv` | `[]` | env vars the agent process receives, as `[{name, value}]` (`runnerExtraEnv` stops at the runner container) |

## Knobs a benchmark may override

| Field | Default |
|---|---|
| `timeout` | `"300"` |
| `deadlineGrace` | `600` |
| `activeDeadlineSeconds` | `""` (derives from `timeout + deadlineGrace`) |
| `runnerArgs` | `/entrypoint.sh; rc=$?; /usr/local/bin/reap-sidecars; exit $rc` |
| `resources.requests` | `{ cpu: 300m, memory: 512Mi, ephemeral-storage: 1Gi }` |
| `resources.limits` | `{ cpu: 2, memory: 2Gi }` |
| `storageOverride` | `""`; `--set storageOverride=24Gi` sets the runner's `ephemeral-storage` limit for one run, over any preset. The request stays the preset's or the default's 1Gi, so a value below it is refused at apply. |
| `outputVolume` | `{}` (→ ephemeral `emptyDir`); set a source to persist `/output`, e.g. `--set outputVolume.hostPath.path=/eval-output` or a `persistentVolumeClaim` |

## Platform composition — *layer via `--overlay` (extra `-f`)*

| Field | Default | Notes |
|---|---|---|
| `serviceAccountName` | `""` | OpenShift sets `anyuid-sa` |
| `sweepId` | `""` | sweep bookkeeping |

## Composition hooks — *for benchmarks with bespoke topology*

Standard benchmarks leave these empty; the bespoke few (osworld-verified, tau-bench,
visualwebarena, webarena) use them to add sidecars, Deployments, and Services.

| Field | Default | Example use |
|---|---|---|
| `initContainers` | `[]` | a wait-for-`<service>` gate |
| `runnerExtraEnv` | `[]` | `EVAL_DESKTOP_HOST` for osworld-verified |
| `gatewayExtraEnv` | `[]` | |
| `runnerExtraVolumeMounts` | `[]` | |
| `extraVolumes` | `[]` | |
| `extraManifests` | `[]` | extra Deployments/Services (full manifests) |

### Custom headers with Bifrost

For an OpenAI-compatible upstream requiring a raw credential header, use:

```yaml
gatewayEnabled: true
gatewayImage: bifrost
apiKeyHeader: RITS_API_KEY
gatewayExtraEnv:
  - name: EVAL_MODEL_API
    value: openai
```

Keep `OPENAI_API_KEY` and `OPENAI_API_BASE` in `eval-secrets`. Bifrost sends
both the configured header and its normal bearer header; the runner receives
neither the custom header setting nor the upstream key. An unset `apiKeyHeader`
keeps ordinary authentication. Other gateway flavors reject the option.
Bifrost's settings API is blocked on the agent-facing port; `/api/health`
remains available. Its private runtime data directory and configuration/database
files are restricted to the gateway user because SQLite also persists static
headers. Rebuild the gateway and model images before using this option.
