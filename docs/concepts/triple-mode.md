# Triple-mode

*Concept · for operators · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 24, [`.agents/src/RULES.md`](../../.agents/src/RULES.md).*

The same evaluation runs on three runtimes. Pick the one that matches your
environment — the benchmark, agent, and model are identical across all three.

| Mode | Wraps | Use it for |
|---|---|---|
| `compose` *(default)* | `docker compose -f containers/benchmarks/<x>/compose.yaml up` | Laptop, full stack (gateway + OTel), fastest iteration |
| `container` | `docker run -e EVAL_MODEL=… <eval-image>-standalone` | CI smoke tests, one-shot runs, single-container footprint |
| `job` | `helm template containers/benchmarks/_chart --set benchmark=<x> \| kubectl apply -f -` | Kubernetes, production-scale parallel runs |

Select the mode with `--mode`:

```bash
eval-containers run aime --task-id 0 --agent codex --mode compose    # default
eval-containers run aime --task-id 0 --agent codex --mode container
eval-containers run aime --task-id 0 --agent codex --mode job
```

## Artifacts per benchmark

Only the compose mode needs a per-benchmark file
([rule 24](../../.agents/benchmarks/RULES.md), enforced by `tests/static/check.rs`):

- `compose.yaml` — the compose stack (`compose` mode)

The `container` and `job` modes both render from a **shared** recipe — no
per-benchmark file:

- `container` mode builds the standalone bundle from the one generic
  `containers/core/standalone.Dockerfile` (`FROM` the lean base + the in-process
  gateway/otelcol/process-compose). The lean base `:latest` is the eval; the
  bundle is the single-container convenience.
- `job` mode renders the shared Helm chart, selected with `--set benchmark=<x>`.
  A benchmark with bespoke topology adds an optional
  `containers/benchmarks/_chart/presets/<x>.yaml`. See [The Helm chart](the-helm-chart.md).

## The mental model

Whichever mode you pick, the wiring is the same: a runner (benchmark + agent), a
gateway (logging model proxy), and otelcol. The mode only changes *who
orchestrates them* — `process-compose` inside one container, a Compose network,
or a Kubernetes Pod. See [Overview](overview.md) and
[Isolation & gateways](isolation-and-gateways.md).
