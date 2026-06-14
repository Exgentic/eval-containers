# Running eval-containers with Podman on Apple Silicon

eval-containers is **Docker-first** — everything is written against the standard
Docker API and you drive it through the `docker` command. Docker Desktop is the
recommended path (see [install.md](install.md)). This guide is the complete
setup for the **alternative**: running on **Podman** on an Apple-Silicon Mac,
which works but has a handful of machine-specific gotchas that aren't obvious.

Once set up, you use `docker` for everything (`docker build`, `docker compose`,
`docker buildx bake`) — never `podman` directly. If you're typing `podman`, you
left the happy path.

## TL;DR

```bash
# 1. Machine (pin the image — newer kernels break Rosetta; see below)
podman machine init --image docker://quay.io/podman/machine-os:5.4
podman machine set --memory 32768 --cpus 10
podman machine ssh "sudo touch /etc/containers/enable-rosetta"
podman machine stop && podman machine start

# 2. docker CLI + compose plugin (brew gives only the client)
brew install docker docker-compose
mkdir -p ~/.docker/cli-plugins
ln -sf /opt/homebrew/opt/docker-compose/bin/docker-compose ~/.docker/cli-plugins/docker-compose

# 3. Point docker (and the test harness) at podman's socket — add to your shell rc
export DOCKER_HOST="unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"

# 4. Build the CLI
cargo build

# Verify
docker version            # reports a running server
docker info | grep -i context
```

## 1. Podman machine

### Pin the machine image (newer kernels break Rosetta)

The default `podman machine init` may pull a Fedora image whose kernel breaks
Rosetta with:

```
rosetta error: unhandled auxillary vector type 29
```

Pin to a known-good machine image:

```bash
podman machine init --image docker://quay.io/podman/machine-os:5.4
```

### Size it

```bash
podman machine set --memory 32768 --cpus 10   # ~half your RAM/cores
```

### Enable Rosetta — REQUIRED

Almost every eval image is `linux/amd64`. Without Rosetta, amd64 falls back to
QEMU, which is **~10× slower** and segfaults on Python extensions (pyarrow,
numpy). Enable it on the machine:

```bash
podman machine ssh "sudo touch /etc/containers/enable-rosetta"
podman machine stop && podman machine start
```

Verify Rosetta is actually active (must print `x86_64`, fast):

```bash
docker run --rm --platform=linux/amd64 python:3.12-slim \
  python -c "import platform; print(platform.machine())"
```

If python-heavy builds SIGILL/segfault, Rosetta isn't on.

## 2. docker CLI + compose plugin

`brew install docker` installs **only the client**. `docker compose` (used by
`eval-containers run` and the compose tests) is a separate plugin:

```bash
brew install docker docker-compose
mkdir -p ~/.docker/cli-plugins
ln -sf /opt/homebrew/opt/docker-compose/bin/docker-compose ~/.docker/cli-plugins/docker-compose
docker compose version   # should report v2.x
```

## 3. DOCKER_HOST — the part that trips everyone

Podman exposes a Docker-compatible socket, but **nothing finds it by default**.
You must export `DOCKER_HOST`, and it matters in **two** places:

```bash
export DOCKER_HOST="unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
```

Add that to your shell rc so it's always set.

- **The `docker` CLI** needs it (or a configured context) to talk to podman.
- **The test harness (testcontainers-rs) needs it too — and ignores the docker
  context.** It hard-looks for `/var/run/docker.sock`, so without `DOCKER_HOST`
  the container tests fail immediately with:
  ```
  Client(Init(SocketNotFoundError("/var/run/docker.sock")))
  ```

## 4. Build the CLI

```bash
brew install rust
cargo build          # produces target/debug/eval-containers
```

## 5. Running the tests

Structural / lint / unit tests need no containers — see [the local testing
guide](../../tests/LOCAL.md). The **container suites** (replay, agents, build,
gateways) run via testcontainers and need two extra env vars on podman:

```bash
export DOCKER_HOST="unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
export TESTCONTAINERS_RYUK_DISABLED=true   # the Ryuk reaper container is flaky under podman

# Cap concurrency — these suites have no internal limit and each test is 2+
# containers; unbounded parallelism thrashes the VM. Rule of thumb: VM_GB / 4.
cargo test --test agents  -- --ignored --test-threads=4
cargo test --test replay  -- --ignored --test-threads=6
```

`TESTCONTAINERS_RYUK_DISABLED=true` skips testcontainers' cleanup sidecar
(Ryuk), which often can't start under podman; containers are still torn down by
the test's own `Drop` handlers.

## 6. Building images / `--local`

For day-to-day dev, build only what you touched (see [LOCAL.md](../../tests/LOCAL.md)).
Two podman-specific notes:

- **Bare `docker build` of an in-repo Dockerfile can 401 — and bake won't save
  you here.** Many Dockerfiles `FROM ${REGISTRY}/core/...:latest`. Until that
  registry is published those bases can only come from the local image store, and
  two podman quirks collide: podman's docker-compat `docker build` *force-pulls* a
  multi-stage stage base (`FROM ... AS x`) from the registry → `401 UNAUTHORIZED`;
  and `docker buildx bake` routes through a BuildKit container that emulates amd64
  with **QEMU, not Rosetta**, so Python-heavy builds segfault (`qemu: uncaught
  target signal 11` installing pyarrow). The only local path that uses Rosetta
  *and* resolves the local bases is **native `podman build`**:
  ```bash
  # Single-FROM benchmark: docker build is fine (buildah → Rosetta; base already local).
  DOCKER_BUILDKIT=0 docker build \
    -t ghcr.io/exgentic/benchmarks/aime:latest containers/benchmarks/aime

  # Multi-stage benchmark (FROM core/<x> AS <x> — the exact-match family, swe-bench):
  # 1. put the tiny FROM-scratch core base into buildah's store (native build);
  podman build -t ghcr.io/exgentic/core/test-exact-match:latest containers/core/test-exact-match
  # 2. build with --platform (so FROM --platform=amd64 matches the local single-arch
  #    base) and --pull=never (don't try the unpublished registry). Rosetta, no QEMU.
  podman build --platform linux/amd64 --pull=never \
    -t ghcr.io/exgentic/benchmarks/gsm8k:latest containers/benchmarks/gsm8k
  ```
  This is the one spot where you must run `podman` directly; once the registry is
  published, plain `docker build` pulls the bases and the workaround goes away.
- **`--model` needs `<provider>/<model>` form** (e.g. `openai/azure/gpt-4.1`),
  not a bare name, or the gateway rejects it:
  ```
  EVAL_MODEL must be of form <provider>/<model> (got: gpt-5.4)
  ```

### BuildKit garbage collection

Cap the build cache so it doesn't grow unbounded:

```bash
podman machine ssh <<'EOF'
sudo tee /etc/containers/containers.conf.d/gc.conf <<CONF
[build]
gc_enabled = true
gc_keep_storage = "20GB"
CONF
EOF
podman machine stop && podman machine start
```

Note: podman's docker-compat socket does **not** support Rosetta under `buildx`.
Single-`FROM` `docker build` works (buildah → Rosetta); multi-stage builds need
the native-`podman build` recipe above; `docker buildx bake` emulates amd64 with
QEMU and segfaults Python builds, so for bake use real Docker.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `rosetta error: unhandled auxillary vector type 29` | machine kernel too new for Rosetta | pin `--image docker://quay.io/podman/machine-os:5.4` (§1) |
| amd64 builds segfault / SIGILL | Rosetta not enabled | `enable-rosetta` (§1) |
| `SocketNotFoundError("/var/run/docker.sock")` in tests | testcontainers ignores docker context | export `DOCKER_HOST` (§3) |
| Ryuk container fails to start | reaper unsupported under podman | `TESTCONTAINERS_RYUK_DISABLED=true` (§5) |
| `docker compose: not found` | client-only install | install + symlink the compose plugin (§2) |
| `401 UNAUTHORIZED` on `docker build`, or `qemu: ... signal 11` on bake | multi-stage `FROM ${REGISTRY}/...` not pulled; bake uses QEMU not Rosetta | native `podman build --platform linux/amd64 --pull=never` (§6) |
| `EVAL_MODEL must be of form <provider>/<model>` | bare model name | pass `<provider>/<model>` (§6) |

## See also

- [install.md](install.md) — Docker Desktop (recommended) setup
- [../../tests/LOCAL.md](../../tests/LOCAL.md) — what to build/test locally and at which level
