# Build test rules

The build category holds the **container build sweep** — one test per
image kind (benchmark, agent, model, core) that actually invokes the
Docker daemon and builds the image from its real Dockerfile.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **testcontainers-rs is mandatory.** Every image MUST be built via
   `GenericBuildableImage::build_image_with()`. Raw `docker build` is
   forbidden per the parent rule 6. The two API-gap carve-outs (rule 6b)
   still apply here: `docker image inspect` for label reads,
   `docker rmi -f` for cleanup.

2. **ImageGuard RAII.** Every built image MUST be owned by an
   `ImageGuard` whose `Drop` impl runs `docker rmi -f`. Without this,
   the podman machine fills within one sweep and every subsequent build
   fails with `no space left on device`.

3. **Core images bootstrap first.** Benchmarks `COPY --from=` the core
   images (`core/entrypoint`, `core/test-exact-match`, `core/litellm`,
   `core/llm-bridge`). The sweep MUST build these first, otherwise
   every benchmark build fails at its first `COPY --from=` step.

## Per-task benchmarks

4. **Per-task benchmarks are skipped by default.** A benchmark whose
   `FROM` line references `${EVAL_TASK_ID}` cannot be built without an
   explicit `--build-arg EVAL_TASK_ID=<value>`. The sweep skips them
   with a visible `⊘` marker unless an entry exists in
   `per_task_build_args()`. These benchmarks are:
   - `swe-bench`, `swe-bench-pro`, `swe-gym`, `swe-lancer`,
     `terminal-bench`, `compilebench`.

5. **Per-task build-arg entries live in the test.** Adding a new
   per-task benchmark to the sweep requires adding a new entry to
   `per_task_build_args()` in `test.rs` with a single known-good task id.

## Known-broken manifest

6. **Every failure MUST be documented or new.** After a sweep run, the
   fleet probe compares the failing set to `tests/build/known-broken.md`.
   Failures within the manifest are yellow; failures outside it are red.

7. **Known-broken entries MUST cite a root cause.** An entry in
   `known-broken.md` MUST explain why the failure is expected on the
   local dev host (e.g. qemu segfault, gated dataset, missing
   credential) AND confirm that CI on `ubuntu-latest` passes it.
   Entries without this citation are drift.

8. **Fix > document.** Documenting a failure in `known-broken.md` is
   a last resort, not a shortcut. If the failure can be fixed, fix it.

## Build env

9. **DOCKER_HOST override.** On macOS with podman, `cargo test` must
   set `DOCKER_HOST` to the podman machine socket. The canonical path
   is `/var/folders/.../T/podman/podman-machine-default-api.sock`. CI
   runners use the default `/var/run/docker.sock`.

10. **Registry prefix.** Built tags MUST use the `eval-build-test-`
    prefix to distinguish sweep artifacts from user-facing images.
