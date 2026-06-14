# Install

*Guide · for operators & contributors · derives from [`README.md`](../../README.md), [`running-tests-locally.md`](running-tests-locally.md).*

What you install depends on how you'll run evals. Install only what your path
needs.

## Prerequisites by use-case

| You want to… | You need |
|---|---|
| Run locally (`compose` / `container`) | Docker Engine + Docker Compose **≥ 2.34** (for `oci://` support) |
| Deploy to Kubernetes (`job`) | `helm` and `kubectl` on PATH, plus cluster access |
| Deploy to OpenShift | the above plus `oc` |
| Build images yourself | Docker with `buildx` (bundled with recent Docker) |
| Use the `eval-containers` CLI | a Rust toolchain (`cargo`) to build it |

The CLI is optional: everything it does is a plain `docker` / `helm` /
`kubectl` command you can run yourself (see [CLI reference](../reference/cli.md)).

## Run with no install beyond Docker

Once the registry is public you won't need to clone the repo:

```bash
echo "OPENAI_API_KEY=sk-..." > .env
export EVAL_BENCHMARK=aime
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f oci://ghcr.io/exgentic/eval-${EVAL_BENCHMARK} up -y --abort-on-container-exit
```

> **Pre-release note.** The `oci://ghcr.io/exgentic/…` registry is the
> published-future shape; the artifacts aren't public yet. For now, clone the
> repo and use `--local` (below) or point `docker compose` at a benchmark's
> `compose.yaml` directly.

## Build and install the CLI

Clone the repo and build the binary with Cargo:

```bash
git clone https://github.com/Exgentic/eval-containers.git
cd eval-containers
cargo build --release
```

The binary lands at `target/release/eval-containers`. Put it on your PATH:

```bash
# adjust to taste
ln -s "$(pwd)/target/release/eval-containers" ~/.local/bin/eval-containers
```

Verify:

```bash
eval-containers --help
eval-containers list            # lists known benchmarks/agents/models
```

## Configure your API key

The CLI auto-loads a `.env` from the working directory (walking up parents):

```bash
echo "OPENAI_API_KEY=sk-..." > .env
```

For cluster deploys, the key lives in a cluster Secret instead — see
[Deploy on Kubernetes](deploy-on-kubernetes.md).

## Next

[Run your first eval](run-your-first-eval.md).

For the local-dev loop (what to build/test at which level), see
[`running-tests-locally.md`](running-tests-locally.md). On Apple Silicon with Podman, see
[Run with Podman on Apple Silicon](podman-on-apple-silicon.md).
