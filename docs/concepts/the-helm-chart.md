# The Helm chart

*Concept · for operators · derives from [`doctrine/benchmarks/RULES.md`](../../doctrine/benchmarks/RULES.md) rules 24/29, [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md).*

In `job` mode, every benchmark deploys through **one shared Helm chart**,
`benchmarks/_chart`. A benchmark contributes only a small `values.yaml`; the
chart renders the otelcol + gateway + runner Job.

```
benchmarks/
  _chart/                 # the one chart — pod shape, defined once
    Chart.yaml
    values.yaml           # defaults for every benchmark
    templates/job.yaml
  aime/values.yaml        # → just:  benchmark: aime
  osworld/values.yaml     # a bespoke one: adds sidecars via composition hooks
```

## Why one chart

The k8s deploy needs three things at once: **reuse** (one pod shape for ~100
benchmarks), **composition** (a few benchmarks add bespoke sidecars), and
**per-run variables** (agent / task / model). Helm is the standard engine that
does all three, and it is first-class on OpenShift. (The previous per-benchmark
Kustomize overlays did reuse + composition but couldn't interpolate variables.)

## What a benchmark overrides

Most benchmarks need a single line:

```yaml
# benchmarks/aime/values.yaml
benchmark: aime
```

The chart's `values.yaml` supplies everything else (image tags, resources,
timeout). The per-run axes — agent, task, model — arrive at deploy time via
`--set` (or the CLI). The bespoke benchmarks (osworld, tau-bench,
visualwebarena, webarena) add sidecars/Deployments through composition hooks
(`initContainers`, `runnerExtraEnv`, `extraManifests`, …).

Full field list: [Chart values reference](../reference/chart-values.md).

## Rendering it

```bash
helm template aime benchmarks/_chart -f benchmarks/aime/values.yaml \
  --set agent=claude-code,task=0 | kubectl apply -f -
```

The `eval-containers run … --mode job` command builds exactly this, mapping each
flag to a `--set`. Platform settings (registry, service account, affinity) layer
in as a second values file via `--overlay` — see
[Deploy on Kubernetes](../guides/deploy-on-kubernetes.md).

## No drift

One chart can't drift from itself — there is no `_base` to keep in sync. CI
renders every benchmark's `values.yaml` through `helm template | kubeconform`
(`tests/helm.rs`) so a broken values file fails the build.
