# Running eval-containers with Podman on Apple Silicon

eval-containers is **Docker-first** — everything is written against the standard
Docker API and you drive it through the `docker` command. Docker Desktop is the
recommended path (see [install.md](install.md)). This guide is the complete
setup for the **alternative**: running on **Podman** on an Apple-Silicon Mac,
which works but has a handful of machine-specific gotchas that aren't obvious.

Once set up you drive everything through `docker`. Since #189 removed all
`FROM --platform=linux/amd64` base-image pins, **`docker buildx bake` builds
native arm64 by default on Apple Silicon** — no QEMU, no pyarrow segfaults. The
`DOCKER_BUILDKIT=0` / classic-build workaround is now edge-case-only (§5, §6).

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

# 5. Build an image — native arm64, bake is fine
docker buildx bake -f containers/docker-bake.hcl \
  -f containers/core/test-exact-match/docker-bake.hcl test-exact-match --load
docker image inspect ghcr.io/exgentic/core/test-exact-match:latest \
  --format '{{.Architecture}}'  # -> arm64
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

### Enable Rosetta (for amd64-only images)

Since #189, most eval images build and run as native arm64. Rosetta is still
needed for **amd64-only images** — `mle-bench` (upstream amd64-only build in
`containers/scripts/build-mle-bench.sh`), swe-bench's `EVAL_BASE_ARCH`
override, or anything you deliberately pull `--platform linux/amd64`.

Without Rosetta, amd64 images fall back to QEMU (~10× slower and segfaults on
Python extensions). Enable it so those exceptions work when you need them:

```bash
podman machine ssh "sudo touch /etc/containers/enable-rosetta"
podman machine stop && podman machine start
```

Verify Rosetta is active (must print `x86_64`, fast):

```bash
docker run --rm --platform=linux/amd64 python:3.12-slim \
  python -c "import platform; print(platform.machine())"
```

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
guide](running-tests-locally.md). The **container suites** (replay, agents, build,
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

> **On podman the cargo `replay` / `build` / `oracle` suites still need
> `DOCKER_BUILDKIT=0`.** The test harness pins `*.platform=linux/amd64` for all
> bake targets because testcontainers runs amd64 containers (`.with_platform("linux/amd64")`),
> so bake still builds linux/amd64 images — which uses QEMU on arm64 and
> segfaults pyarrow. Prefix `DOCKER_BUILDKIT=0`:
> `DOCKER_BUILDKIT=0 cargo test --test replay` (and `--test build`). The fast
> suites (`check`, `compose`, `dockerfile_inspection`, `task_inspection`, `helm`)
> need no builder and are unaffected.

## 5a. Why `docker buildx bake` can't use Rosetta on podman (historical)

This section explains the pre-#189 QEMU issue; it is now edge-case-only for the
test suites above. `docker buildx bake` without a `--platform` override builds
native arm64 since #189 dropped the `FROM --platform=linux/amd64` base pins.

`docker buildx bake` runs the build in a **BuildKit daemon**, and BuildKit does
**not** use the kernel's binfmt handler for foreign-arch `RUN` steps — it injects
its own bundled emulator (`/usr/bin/buildkit-qemu-x86_64`, bind-mounted into each
build sandbox as `/dev/.buildkit_qemu_emulator`). So:

- **Stock containerized BuildKit** emulates amd64 with **QEMU** when the platform
  is linux/amd64 → pyarrow/numpy segfault. Bind-mounting `/mnt/rosetta` + the host
  `binfmt_misc` into the buildkitd changes nothing — BuildKit ignores them and uses
  its own qemu.
- **Replacing** `buildkit-qemu-x86_64` with the Rosetta interpreter does get
  BuildKit to invoke Rosetta, but Rosetta then refuses. The sandbox strips what
  Rosetta needs to detect the vz environment.

The fix (still valid when amd64 is explicitly needed) is to **not use BuildKit**:
classic `docker build` routes to buildah, which *does* go through the kernel
`rosetta` handler. The cargo test suites do this under **`DOCKER_BUILDKIT=0`**
when amd64 images are required: the harness reads the build graph from
`docker buildx bake --print` and builds each image with classic `docker build` →
Rosetta (§5, §6).

## 6. Building images / `--local`

Since #189, **`docker buildx bake` is the normal local build on Apple Silicon**.
It produces native arm64 images with no QEMU involved:

```bash
# Build any target natively (arm64)
docker buildx bake -f containers/docker-bake.hcl \
  -f containers/core/test-exact-match/docker-bake.hcl test-exact-match --load
```

### Docker Hub rate limits (429)

On first build, the most common friction is Docker Hub pulling base images
(e.g. `python:3.12-slim`) and hitting the unauthenticated pull limit:

```
toomanyrequests: You have reached your pull rate limit.
```

Fix: log in to Docker Hub (`docker login`), or use the ECR public mirror which
has no rate limit:

```bash
# Prefix your build with the ECR mirror (no login required)
docker pull public.ecr.aws/docker/library/python:3.12-slim
docker tag  public.ecr.aws/docker/library/python:3.12-slim python:3.12-slim
```

### amd64-only exceptions

A small number of images are genuinely amd64-only and still require the classic
build path:

- **`containers/scripts/build-mle-bench.sh`** — upstream openai/mle-bench is
  amd64-only; the script already sets `--platform=linux/amd64`.
- **swe-bench** — uses `EVAL_BASE_ARCH` (default `x86_64`) to select the
  upstream epoch-research base; build with `podman build --platform linux/amd64`.
- **Anything you pull with `--platform linux/amd64` explicitly** — uses
  Rosetta (§1) instead of QEMU when built with classic `docker build`.

For images that `FROM ${REGISTRY}/core/...:latest` where the base is not yet
published, two podman quirks collide: podman's docker-compat `docker build`
force-pulls a multi-stage stage base from the registry → `401 UNAUTHORIZED`; and
bake routes through BuildKit which uses QEMU for amd64. The fallback:

```bash
# Multi-stage benchmark that needs amd64 and an unpublished base:
# 1. Build the core base into buildah's local store
podman build -t ghcr.io/exgentic/core/test-exact-match:latest \
  containers/core/test-exact-match

# 2. Build with --platform + --pull=never (Rosetta, no QEMU)
podman build --platform linux/amd64 --pull=never \
  -t ghcr.io/exgentic/benchmarks/gsm8k:latest containers/benchmarks/gsm8k
```

- **Gated benchmarks (`gaia`, `hle`, `flores200`) need a build secret** —
  `DOCKER_BUILDKIT=0 docker build` can't pass `--mount=type=secret`. Use native
  `podman build --secret` or Docker Desktop:
  ```bash
  HF_TOKEN=hf_… podman build --platform linux/amd64 --pull=never \
    --secret id=HF_TOKEN,env=HF_TOKEN \
    -t ghcr.io/exgentic/benchmarks/gaia:latest containers/benchmarks/gaia
  ```
- **`--model` needs `<provider>/<model>` form** (e.g. `openai/azure/gpt-4.1`),
  not a bare name, or the gateway rejects it:
  ```
  EVAL_MODEL must be of form <provider>/<model> (got: gpt-5.4)
  ```

### Recording a replay fixture for a new benchmark

`eval-containers build eval <name> --agent <a>` invokes `docker buildx bake`,
which on Apple Silicon now builds native arm64. To record a replay fixture:

```bash
# 1. Pull deps that are already published
for img in agents/codex models/bifrost core/otel core/runtime-bundle; do
  docker pull ghcr.io/exgentic/$img:latest
done

# 2. Build the benchmark image (native arm64)
docker buildx bake -f containers/docker-bake.hcl \
  -f containers/benchmarks/<name>/docker-bake.hcl <name> --load

# 3. Run one task — needs OPENAI_API_KEY + OPENAI_API_BASE in .env
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=openai/azure/gpt-5.4 \
  docker compose -f containers/benchmarks/<name>/compose.yaml up --abort-on-container-exit

# 4. Extract the trajectory from the named volume (NOT a host path)
docker run --rm -v <name>_output:/output:ro alpine \
  cat /output/traces.jsonl > tests/run/replay/fixtures/<name>-0-codex.traces.jsonl

# 5. Register the fixture in tests/run/replay/test.rs (replay_test! macro) and ship
```

The named volume is the gotcha — `find output/` returns nothing because compose
mounts `output:/output` (declared in `compose/services.yaml`), not a bind
mount. Use `docker run -v <name>_output:/output:ro` to read it.

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

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `rosetta error: unhandled auxillary vector type 29` | machine kernel too new for Rosetta | pin `--image docker://quay.io/podman/machine-os:5.4` (§1) |
| amd64 builds segfault / SIGILL | Rosetta not enabled | `enable-rosetta` (§1) |
| `SocketNotFoundError("/var/run/docker.sock")` in tests | testcontainers ignores docker context | export `DOCKER_HOST` (§3) |
| Ryuk container fails to start | reaper unsupported under podman | `TESTCONTAINERS_RYUK_DISABLED=true` (§5) |
| `docker compose: not found` | client-only install | install + symlink the compose plugin (§2) |
| `toomanyrequests` / 429 on pull | Docker Hub rate limit | `docker login` or use ECR public mirror (§6) |
| `401 UNAUTHORIZED` on `docker build` of multi-stage image | podman force-pulls unpublished base | `podman build --platform linux/amd64 --pull=never` (§6) |
| `cargo test --test replay`/`build` segfaults in `pyarrow` (`bootstrap_core_bases`) | test harness pins linux/amd64, bake uses QEMU for that target | prefix `DOCKER_BUILDKIT=0` (§5) |
| `EVAL_MODEL must be of form <provider>/<model>` | bare model name | pass `<provider>/<model>` (§6) |

## See also

- [install.md](install.md) — Docker Desktop (recommended) setup
- [running-tests-locally.md](running-tests-locally.md) — what to build/test locally and at which level
