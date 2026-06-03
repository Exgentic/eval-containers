# Triple-mode

*Concept · for operators · derives from [`doctrine/benchmarks/RULES.md`](../../doctrine/benchmarks/RULES.md) rule 24, [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md).*

The same evaluation runs on three runtimes. Pick the one that matches your
environment — the benchmark, agent, and model are identical across all three.

| Mode | Wraps | Use it for |
|---|---|---|
| `compose` *(default)* | `docker compose -f benchmarks/<x>/compose.yaml up` | Laptop, full stack (gateway + OTel), fastest iteration |
| `container` | `docker run -e EVAL_MODEL=… <eval-image>` | CI smoke tests, one-shot runs, minimal footprint |
| `job` | `helm template benchmarks/_chart -f benchmarks/<x>/values.yaml \| kubectl apply -f -` | Kubernetes, production-scale parallel runs |

Select the mode with `--mode`:

```bash
eval-containers run aime --task-id 0 --agent codex --mode compose    # default
eval-containers run aime --task-id 0 --agent codex --mode container
eval-containers run aime --task-id 0 --agent codex --mode job
```

## Three artifacts per benchmark

Every benchmark carries exactly three deploy artifacts — this is the
"triple-mode" invariant ([rule 24](../../doctrine/benchmarks/RULES.md), enforced
by `tests/sanity/check.rs`):

- `container.Dockerfile` — the single-container image (`container` mode)
- `compose.yaml` — the compose stack (`compose` mode)
- `values.yaml` — Helm values over the shared chart (`job` mode)

The container and compose modes wrap unmodified Docker. The job mode renders one
shared Helm chart — see [The Helm chart](the-helm-chart.md).

## The mental model

Whichever mode you pick, the wiring is the same: a runner (benchmark + agent), a
gateway (logging model proxy), and otelcol. The mode only changes *who
orchestrates them* — `process-compose` inside one container, a Compose network,
or a Kubernetes Pod. See [Overview](overview.md) and
[Isolation & gateways](isolation-and-gateways.md).
