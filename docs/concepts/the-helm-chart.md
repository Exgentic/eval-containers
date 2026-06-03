# The Helm chart

*Concept · for operators · derives from [`doctrine/benchmarks/RULES.md`](../../doctrine/benchmarks/RULES.md) rules 24/29, [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md).*

In `job` mode, every benchmark deploys through **one shared Helm chart**,
`benchmarks/_chart`. A benchmark is selected by name (`--set benchmark=<x>`);
the chart renders the otelcol + gateway + runner Job. A benchmark with bespoke
topology adds an optional preset file inside the chart — standard benchmarks
contribute nothing, so the published chart is self-contained.

```
benchmarks/
  _chart/                 # the one chart — pod shape, defined once
    Chart.yaml
    values.yaml           # defaults for every benchmark
    presets/              # optional per-benchmark topology, bundled in the chart
      osworld.yaml        #   adds sidecars/Deployments via composition hooks
      tau-bench.yaml
    templates/job.yaml
  aime/                   # standard benchmark — no preset; just --set benchmark=aime
```

## Why one chart

The k8s deploy needs three things at once: **reuse** (one pod shape for ~100
benchmarks), **composition** (a few benchmarks add bespoke sidecars), and
**per-run variables** (agent / task / model). Helm is the standard engine that
does all three, and it is first-class on OpenShift. (The previous per-benchmark
Kustomize overlays did reuse + composition but couldn't interpolate variables.)

## What a benchmark overrides

Most benchmarks override nothing — naming one with `--set benchmark=aime` is
enough, and the chart's `values.yaml` supplies everything else (image tags,
resources, timeout). The per-run axes — agent, task, model — arrive at deploy
time via `--set` (or the CLI). The bespoke benchmarks (osworld, tau-bench,
visualwebarena, webarena) ship a `presets/<name>.yaml` in the chart that adds
sidecars/Deployments through composition hooks (`initContainers`,
`runnerExtraEnv`, `extraManifests`, …); the chart overlays it automatically when
that benchmark is selected.

Full field list: [Chart values reference](../reference/chart-values.md).

## Rendering it

```bash
helm template aime benchmarks/_chart --set benchmark=aime \
  --set agent=claude-code,task=0 | kubectl apply -f -
```

The `eval-containers run … --mode job` command builds exactly this, mapping each
flag to a `--set`. Platform settings (registry, service account, affinity) layer
in as a second values file via `--overlay` — see
[Deploy on Kubernetes](../guides/deploy-on-kubernetes.md).

## No drift

One chart can't drift from itself — there is no `_base` to keep in sync. CI
renders every benchmark through `helm template --set benchmark=<x> | kubeconform`
(`tests/helm.rs`) so a broken chart or preset fails the build.
