# Docker Capabilities: Considered and Declined

Features we evaluated and intentionally chose not to use, with rationale. Each section includes the case for adoption so the decision can be revisited with full context.

## Compose `include`

**What it does:** Import another compose file, merging services. Reduces repetition across compose files.

**Case for:** 14 compose files share an identical model service block (~8 lines) and similar eval service boilerplate (~15 lines). When the model service pattern changes (e.g., adding a health check, changing the port, adding a logging sidecar), you update one file instead of 14. `docker compose publish` bundles included files into the OCI artifact, so published compose files remain self-contained for consumers. It's native Compose, not a custom tool. As Dock grows to 50+ benchmarks, the maintenance burden of duplicated YAML grows linearly.

**Why not:** Each compose file is currently 40-80 lines. The duplication is small and the benefit of reading one file to understand the full setup is worth more than DRY savings. `include` introduces a dependency between files — the compose file is no longer understandable in isolation during development. Relative paths (`../base.yaml`) are fragile. The risk of implicit changes (modify base, break a benchmark) outweighs the cost of updating 14 files. Users can still `include` our published compose files in their own compositions without us using `include` internally.

**Revisit when:** Dock exceeds ~30 benchmarks, or the shared boilerplate grows beyond the model service.

## Compose `extends`

**What it does:** Inherit a service definition from another file or within the same file.

**Case for:** Could define a base eval service with common settings (volumes, cap_drop, security_opt, depends_on model) and have each benchmark extend it with just the image name, environment, and resources. More granular than `include` — you override at the service level, not the file level.

**Why not:** Same readability tradeoff as `include`. You need to read two files to understand what runs. The common eval settings (volumes, security) are 6 lines — not enough to justify the indirection. `extends` doesn't work across `docker compose publish` boundaries as cleanly as `include`.

**Revisit when:** Same as `include`.

## Docker Bake

**What it does:** Declarative build orchestration. Parallel multi-target builds, matrix builds, target inheritance, registry cache export/import.

**Case for:** Dock's build model is a dependency graph: eval image depends on bench image + agent image. Bake handles this natively — declare targets with dependencies, build in parallel, share cache. For CI, Bake can build the entire benchmark × agent matrix in one command with `--set '*.platform=linux/amd64'` and push everything to the registry. Registry cache export (`--cache-to type=registry`) means CI never cold-builds. Compose v5 delegates builds to Bake internally, so it's becoming the standard anyway. The HCL config could replace most of `build.rs`.

**Why not:** Most users build one eval at a time. For single builds, `docker build` with layer caching is fast and simple. Bake adds HCL as a config format — another thing to learn and maintain. The auto-build logic in `build.rs` (check if image exists, build dependencies first) is 30 lines of Rust that do exactly what we need. Bake is more powerful but that power isn't needed for sequential single-target builds. The CLI already shells out to `docker build` — the Docker equivalent is obvious and auditable.

**Revisit when:** CI build times become a bottleneck, or when building the full matrix for a release takes too long. At that point, add a `docker-bake.hcl` for CI use only — don't change the user-facing CLI.

## Docker Scout

**What it does:** Built-in vulnerability scanning for images.

**Why not now:** Good idea for later. Not core to the build system. Can be added to CI without any Dock code changes.

## Docker Debug

**What it does:** Attach a debugging toolbox to any running container without modifying the image.

**Why not now:** Useful but orthogonal. Users can run `docker debug` themselves on any eval container. No Dock integration needed.

## Compose `profiles`

**What it does:** Tag services with profiles, selectively activate with `--profile`. e.g., `--profile monitoring` to add Grafana.

**Case for:** Could add optional services like Langfuse (tracing), Prometheus (metrics), or a debug shell as profiled services. Users opt in with `--profile monitoring` without maintaining a separate compose file. Benchmarks that need GPUs could have a `gpu` profile.

**Why not now:** Our compose files define exactly what a benchmark needs — nothing optional. Optional services (monitoring, debugging) are user concerns, best added via `include` in their own compose file. Profiles add complexity to compose files that most users won't use. If we add Langfuse integration later, profiles would be the right mechanism.

**Revisit when:** Dock adds first-party observability or debugging services.

## Compose `models` element (Spec v5)

**What it does:** Define AI/ML models as OCI artifacts directly in Compose.

**Why not:** We handle models as Docker images (LiteLLM proxy with pre-configured routing). The `models` element is designed for packaging model weights as OCI artifacts, which is a different use case. Our model images are lightweight proxies, not weight files.

## OCI artifacts via `oras`

**What it does:** Push/pull arbitrary files to OCI registries.

**Why not:** Docker Compose natively supports `docker compose publish` and `oci://` references for compose files. No need for a separate tool or dependency.

## Image labels for compose storage

**What it does:** Store compose YAML as a base64-encoded image label, retrievable via `docker inspect`.

**Why not:** Labels are meant for short metadata. Base64-encoding multi-line YAML into a label is a hack. `docker compose publish` is the proper mechanism.

---

## Features we DO use

- **`docker compose publish` / `oci://`** — compose files stored in and pulled from OCI registries
- **BuildKit heredocs** (`# syntax=docker/dockerfile:1`) — inline scripts in Dockerfiles without external files
- **BuildKit cache mounts** — faster dependency installs in agent images
- **Multi-stage `COPY --from`** — combine benchmark + agent images without runtime installation
- **`include_str!()`** — embed combination Dockerfile in the Rust binary (compile-time)
- **`DOCK_REGISTRY` env var** — same commands work against local `registry:2` and `ghcr.io`
