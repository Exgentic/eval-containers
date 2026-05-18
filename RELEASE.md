# Releasing Eval Containers Images

**Status:** Production release flow
**Date:** April 2026

This document covers how Eval Containers images get built, tagged, and pushed to the registry in bulk. For the local dev loop (build one benchmark, run one replay), see [tests/LOCAL.md](tests/LOCAL.md) instead.

## Principle

**CI builds the fleet. Humans build one thing at a time.**

Releasing means producing 77 benchmark images + 11 agent images, all tagged, labeled, and pushed to `quay.io/eval-containers`. That's a fleet build. The right tool is [Docker Bake](https://docs.docker.com/build/bake/) — Docker's native declarative multi-image build tool.

## The plan

There is no committed bake file. `scripts/bake-plan.sh` emits a bake plan as JSON on stdout, generated from the filesystem — one target per directory in `benchmarks/` and `agents/`. The filesystem is the single source of truth; adding a benchmark is one `mkdir`, no list to regenerate.

Every target gets:
- `platforms: ["linux/amd64"]`
- `eval.type` label (benchmark or agent)
- OCI labels: `image.source`, `image.revision` (git sha), `image.created` (build date)
- Tag: `${REGISTRY}/{benchmarks|agents}/<name>:${TAG}`

Three groups: `benchmarks`, `agents`, `default` (both).

## Commands

Pipe the plan into bake via process substitution:

```bash
# Dry-run: resolved plan as JSON, build nothing
docker buildx bake -f <(scripts/bake-plan.sh) --print

# Lint every Dockerfile, build nothing
docker buildx bake -f <(scripts/bake-plan.sh) --check

# Build everything locally (no push)
docker buildx bake -f <(scripts/bake-plan.sh)

# Build + push to quay.io/eval-containers (the actual release step)
docker buildx bake -f <(scripts/bake-plan.sh) --push

# One target by name
docker buildx bake -f <(scripts/bake-plan.sh) bench-aime

# One group
docker buildx bake -f <(scripts/bake-plan.sh) benchmarks
docker buildx bake -f <(scripts/bake-plan.sh) agents
```

Overrides (exported env vars that `bake-plan.sh` reads):

```bash
REGISTRY=ghcr.io/elron-staging scripts/bake-plan.sh | docker buildx bake -f- --push
TAG=v1.2.0                     scripts/bake-plan.sh | docker buildx bake -f- --push
```

## Dependencies

`scripts/bake-plan.sh` needs `bash`, `find`, and `jq`. All standard on macOS (with `brew install jq`) and every Linux CI runner.

## CI workflow

[`.github/workflows/release.yml`](.github/workflows/release.yml) runs bake on every push to `main` (tag: `latest`) and every `v*` tag (tag: the git tag). It calls `scripts/bake-plan.sh` with `GIT_SHA` and `BUILD_DATE` set, then `bake --push`es the result.

## Podman note

Bake requires `buildx`, which podman's docker-compat shim doesn't ship. If you need to run the fleet build locally against podman, install real Docker CLI alongside podman:

```bash
brew install docker-buildx
export DOCKER_HOST=unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')
docker buildx bake -f <(scripts/bake-plan.sh) --print
```

But in practice, you probably shouldn't. Let CI do fleet builds. Humans build one thing at a time.

## What Bake does not do

- **Run tests.** Use `cargo test --test compose` and `cargo test --test replay`.
- **Run agents.** Use `docker compose up` or `eval-containers run`.
- **Verify labels post-build.** That's `tests/build.rs`.

## References

- [Docker Bake docs](https://docs.docker.com/build/bake/)
- [tests/LOCAL.md](tests/LOCAL.md) — local dev loop
- [tests/RULES.md](tests/RULES.md) — what tests MUST do
