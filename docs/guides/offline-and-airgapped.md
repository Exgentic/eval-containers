# Run offline or airgapped

*Guide · for operators · derives from [`.agents/compose/RULES.md`](../../.agents/compose/RULES.md), [`install.md`](install.md).*

`oci://` compose references need Docker Compose ≥ 2.34 and network access to the
registry. If you're on older Docker, behind a firewall, or fully airgapped, run
from a flattened compose file — and, for hosts with no registry access, from a
saved image bundle.

## Flatten the compose file

On a machine with network and a recent Docker, resolve the `oci://` reference
into a plain compose file once:

```bash
EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f oci://ghcr.io/exgentic/eval-aime config > aime.compose.yaml
```

Copy `aime.compose.yaml` to the target host and run it with any Compose
version — no `oci://` support needed:

```bash
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f aime.compose.yaml up --abort-on-container-exit
```

> **Pre-release note.** The `oci://ghcr.io/exgentic/…` registry is the
> published-future shape; the artifacts aren't public yet. Until then, flatten
> from a clone with `--local`: `docker compose -f
> containers/benchmarks/aime/compose.yaml config > aime.compose.yaml`.

## Fully airgapped: bundle the images

The flattened file still pulls images by reference. For a host with no registry
access at all, save those images on a connected machine and load them on the
target. Derive the exact list from the flattened compose file — no guessing:

```bash
# Connected machine: save every image the run needs into one archive
docker save $(docker compose -f aime.compose.yaml config --images) \
  | gzip > aime-bundle.tar.gz

# Airgapped host: load the images, then run from the flattened compose file
docker load < aime-bundle.tar.gz
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f aime.compose.yaml up --abort-on-container-exit
```

## See also

- [Install](install.md) — Docker Compose version requirements per use-case.
- [Run your first eval](run-your-first-eval.md) — the normal online path.
