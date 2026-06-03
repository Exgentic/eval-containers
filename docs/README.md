# Eval Containers — Documentation

Human-facing docs for running, deploying, and extending Eval Containers.

> This is the *explanation*. The binding rules live in [`doctrine/`](../doctrine/);
> when docs and doctrine disagree, doctrine wins. See
> [`doctrine/docs/RULES.md`](../doctrine/docs/RULES.md) for what governs these pages.

Eval Containers runs AI-agent evaluations as plain container artifacts. One
evaluation is **one benchmark + one agent + one model**, and it runs the same
way on a laptop, in CI, or on a Kubernetes cluster.

## Start here

- New to the project? Read [Concepts → Overview](concepts/overview.md).
- Want to run something now? [Install](guides/install.md) →
  [Run your first eval](guides/run-your-first-eval.md).
- Going to a cluster? [Deploy on Kubernetes](guides/deploy-on-kubernetes.md).

## Concepts — *what it is and why*

- [Overview](concepts/overview.md) — the model: images as the product, three axes
- [Triple-mode](concepts/triple-mode.md) — the same eval as container / compose / k8s job
- [Isolation & gateways](concepts/isolation-and-gateways.md) — how trajectories stay honest
- [The Helm chart](concepts/the-helm-chart.md) — one chart, `--set benchmark=<x>` to select; optional per-benchmark preset

## Guides — *how to do a task*

- [Install](guides/install.md)
- [Run your first eval](guides/run-your-first-eval.md)
- [Deploy on Kubernetes](guides/deploy-on-kubernetes.md)
- [Deploy on OpenShift](guides/deploy-on-openshift.md)
- [Add a benchmark](guides/add-a-benchmark.md)
- [Add an agent](guides/add-an-agent.md)
- [Add a model](guides/add-a-model.md)

## Reference — *exact flags, vars, values*

- [CLI](reference/cli.md) — `eval-containers` commands and flags
- [Environment variables](reference/env-vars.md) — the `EVAL_*` namespace
- [Chart values](reference/chart-values.md) — `benchmarks/_chart` values
