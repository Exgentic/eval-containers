# Triple-mode

*Concept · for operators · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 24, [`.agents/src/RULES.md`](../../.agents/src/RULES.md).*

The same evaluation runs on three runtimes. Pick the one that matches your
environment — the benchmark, agent, and model are identical across all three.

| Mode | Wraps | Use it for |
|---|---|---|
| `compose` *(default)* | `docker compose -f benchmarks/<x>/compose.yaml up` | Laptop, full stack (gateway + OTel), fastest iteration |
| `container` | `docker run -e EVAL_MODEL=… <eval-image>` | CI smoke tests, one-shot runs, minimal footprint |
| `job` | `helm template benchmarks/_chart --set benchmark=<x> \| kubectl apply -f -` | Kubernetes, production-scale parallel runs |

Select the mode with `--mode`:

```bash
eval-containers run aime --task-id 0 --agent codex --mode compose    # default
eval-containers run aime --task-id 0 --agent codex --mode container
eval-containers run aime --task-id 0 --agent codex --mode job
```

## Artifacts per benchmark

The container and compose modes each need one file in the benchmark's dir
([rule 24](../../.agents/benchmarks/RULES.md), enforced by `tests/sanity/check.rs`):

- `container.Dockerfile` — the single-container image (`container` mode)
- `compose.yaml` — the compose stack (`compose` mode)

The `job` mode renders one shared Helm chart, selected with `--set benchmark=<x>`
— no per-benchmark file required. A benchmark with bespoke topology adds an
optional `benchmarks/_chart/presets/<x>.yaml`. See
[The Helm chart](the-helm-chart.md).

## The mental model

Whichever mode you pick, the wiring is the same: a runner (benchmark + agent), a
gateway (logging model proxy), and otelcol. The mode only changes *who
orchestrates them* — `process-compose` inside one container, a Compose network,
or a Kubernetes Pod. See [Overview](overview.md) and
[Isolation & gateways](isolation-and-gateways.md).
