# Running evals on a local kind cluster

[kind](https://kind.sigs.k8s.io/) runs a Kubernetes cluster on your laptop, so
you can develop against the same Job surface as a real cluster without one. It's
for a fast local dev loop — build, submit, debug, repeat — that stays close to
how evals actually run in production.

The model in one line: **a dataset eval is one [Indexed Job](https://kubernetes.io/docs/tasks/job/indexed-parallel-processing-static/)** — each example is a completion index, Kubernetes fans them out, caps concurrency, retries per-example, and cleans up. This is the same Job surface as [OpenShift](../oc/README.md); the difference is *where the images come from*. On kind you build every image **on your laptop** and move it into the cluster — there is no registry to pull from. The scripts here are thin wrappers over four standard tools:

| tool | role |
|------|------|
| `eval-containers` (the repo CLI) | build images locally (`build bench/agent/model/eval`) |
| `kind` | load host-built images into the node's containerd; `create.sh` provisions the cluster |
| `helm` | render the Job from `containers/benchmarks/_chart` |
| `kubectl` | apply, watch |

Run the scripts from the repo root (paths below are `./deploy/kind/…`). With a
**rootless Podman** engine rather than Docker, `export KIND_EXPERIMENTAL_PROVIDER=podman`
first — every `kind` command (and the scripts, which shell out to `kind`) needs
it, and `kind create` otherwise fails with `requires setting systemd property
"Delegate=yes"`. See [`docs/guides/deploy-on-kind.md`](../../docs/guides/deploy-on-kind.md)
for the one-time cgroup-delegation setup.

## Scripts (all of `deploy/kind/`)

| script | what |
|--------|------|
| `create.sh` | provision the cluster (if absent) **and** the `eval-secrets` Secret from `OPENAI_API_KEY` + `OPENAI_API_BASE` in the environment. Run once before `run.sh`. |
| `run.sh`  | build on the host → load into kind → submit **one** eval. `--dataset` (or `--dataset-size N`) → an Indexed Job over the dataset; omit it for a single-`--task` debug run. Errors if the cluster is absent. |
| `_lib.sh` | shared defaults (cluster/registry/output) + the `kind_reload` (conditional load) and `job_refs` (image-ref) helpers. |

`create.sh` carries `--cluster <name>` (default `eval`), `--namespace <ns>`,
`--output-dir <p>` (host dir bind-mounted into the node at `/eval-output` so
results land on your host filesystem — default `./eval-output`, created if
absent; takes effect only at cluster creation, so pair with `--recreate` to
remount), `--recreate` (delete an existing cluster of this name and rebuild it),
`--dry-run`, and `--help`.

`run.sh` carries `--cluster <name>` (default `eval`), `--rebuild` (force rebuild
+ reload), `--no-build`, `--no-run` (build only), `--rerun` (delete an existing
Job first — Jobs are immutable once run), `--output-path <p>` (node dir for
`/output`, default `/eval-output`), `--watch`, `--dry-run`, and `--help`.

## Quickstart

```bash
# one-time: provision the cluster + the eval-secrets Secret. The gateway needs
# upstream credentials (reachable from your machine); create.sh reads them from
# the environment.
OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-endpoint ./deploy/kind/create.sh

# single example, for debugging
./deploy/kind/run.sh --benchmark aime --agent codex --model bifrost --task 0 --watch

# a whole dataset as an Indexed Job (parallelism defaults to 2 on a laptop)
./deploy/kind/run.sh --benchmark aime --agent codex --model bifrost --dataset --parallelism 4 --watch

kubectl --context kind-eval get jobs                    # run progress
cat ./eval-output/aime/codex/bifrost/0/result.json      # results on the host (default bind-mount)
kind delete cluster --name eval                         # teardown
```

By default `create.sh` bind-mounts `./eval-output` into the node at the
`/eval-output` hostPath (`--output-dir` overrides the host dir), so results land
on your *host* filesystem — `cat ./eval-output/aime/codex/bifrost/0/result.json`.
The bind-mount is fixed at cluster creation; change `--output-dir` on an existing
cluster by re-running with `--recreate`. Results also remain readable inside the
node with `docker exec <cluster>-control-plane cat /eval-output/…`.

## Private-CA upstreams (e.g. IBM internal)

If your `OPENAI_API_BASE` is served behind a **private CA** — an internal endpoint
whose TLS cert chains to a corporate root that your laptop trusts (via the system
keychain) but a pod does not — the gateway pod fails TLS verification even though
the host can reach the endpoint (`curl` exit 60 / Go `x509: certificate signed by
unknown authority`). The pod's trust store ships only public roots.

Fix it at deploy time — the CA stays **on the cluster** as a ConfigMap and is
never baked into (or pushed with) any image. Point `EVAL_UPSTREAM_CA` at a PEM of
the extra CA cert(s) when you provision:

```bash
# macOS: export the corporate root (+ intermediate) from the keychain to a PEM.
# The endpoint's leaf chains to these; the served chain omits the root, so pull it
# from the keychain where your IT already installed it.
security find-certificate -a -c "IBM Internal Root CA"         -p > ibm-ca.pem
security find-certificate -a -c "IBM INTERNAL INTERMEDIATE CA" -p >> ibm-ca.pem

EVAL_UPSTREAM_CA=./ibm-ca.pem \
  OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-internal-endpoint \
  ./deploy/kind/create.sh
```

`create.sh` validates the file is CA certs (and refuses a private key), then stores
it as the `eval-upstream-ca` ConfigMap. On the next `run.sh`, if that ConfigMap
exists it is mounted into the gateway at `/etc/eval-ca/ca.pem` with
`SSL_CERT_FILE` pointed at it — the Go gateway **appends** it to the public roots,
so both public and private upstreams keep verifying. Omit `EVAL_UPSTREAM_CA` and
nothing changes: no ConfigMap, no mount, public-roots-only trust as before.

## How images reach the cluster (and the `:latest` trap)

`run.sh` builds `bench`, `agent`, `model` (the gateway), and the combined `eval`
(runner) image locally, plus `otel` via `docker buildx bake otel --load` (otel
has no `build` subcommand). The `eval` build uses `--no-pull` so it wires the
just-built bench+agent from the local BuildKit store rather than doing a registry
manifest check — required on arm64.

The chart runs `imagePullPolicy: IfNotPresent` on `:latest` tags. In kind,
"present" means loaded into the node's containerd — so once a `:latest` is cached
there the node reuses it forever, and a rebuild is served **only after the stale
node copy is evicted**. `run.sh` handles this per image: it compares the host
image id against the node's cached id and, when they differ, does `crictl rmi`
then `kind load`; when they match it skips the (expensive) reload. So unchanged
components are skipped and a changed one always reloads.

**Preset sidecars.** Some benchmarks (`tau-bench`, `osworld`, `webarena`, …) add
sidecar images in `containers/benchmarks/_chart/presets/<benchmark>.yaml`:

- **Public third-party images** (`mitmproxy`, `busybox`, `am1n3e/*`,
  `happysixd/*`, `shivakrishnareddyma225/*`) — the node's kubelet pulls these
  itself at pod-start; `run.sh` leaves them alone (an offline or rate-limited run
  is then an expected failure, not a script bug).
- **Private `ghcr.io/exgentic/*` images** (e.g. tau-bench's `tau-bench-bridge`) —
  the node cannot pull these; they are loaded like the main images, but `run.sh`
  only builds the four core images, so build any such sidecar on the host first.

## Concurrency

Unlike OpenShift there is no Kueue and no cluster autoscaler: the Job's
`parallelism` is the only concurrency cap, and a kind cluster is bounded by
whatever CPU/memory your container engine gives its single node. Excess indices
sit `Pending` rather than running, so a dataset run with too-high parallelism
just queues. `run.sh` defaults `--parallelism 2` for a dataset run when you don't
set it; raise it if your node has the headroom.

## Cluster requirements

```bash
kind version && kubectl version --client && helm version && docker version   # all present
```

- `kind`, `kubectl`, `helm`, `docker`, and the repo CLI (`target/release/eval-containers`, auto-added to PATH) available.
- An `eval-secrets` Secret (`OPENAI_API_KEY` + `OPENAI_API_BASE`) in the target namespace — the gateway reads it.
- No PVC, no `anyuid` SCC, no Kueue (those are OpenShift concerns). kind ships a
  current Kubernetes, so `completionMode: Indexed` and `--retry`
  (`backoffLimitPerIndex`) are always available.
