# Static-stage test rules

The **static** stage holds **fast mechanical gates** that run on every plain
`cargo test` — no `--ignored` flag, no container daemon, no network. These are
the gates a contributor's local build and every CI job depend on for green/red
signal in seconds. This is the stage that implements the **Sanity** phase of
[VERIFY.md](VERIFY.md) (the term "sanity" elsewhere in the doctrine refers to
that verification phase; this crate is its home).

The stage is split by tool: artifact-shaped structure (Dockerfile / compose /
helm / CVE) is owned by standard-tool policy beside this file — `policy/`
(conftest/OPA) and `security/` (trivy) — while the Rust targets here hold only
the residual no standard tool expresses: cross-file repo invariants (`check`),
trajectory-data analysis (`task_inspection`), and the procedural pip/npm pin
walk (`dockerfile_inspection`). Lives in the dependency-light
`eval-containers-tests-static` crate so the every-PR gate compiles cheaply.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Must run offline.** Sanity tests MUST NOT make network calls, pull
   images, or contact a Docker daemon. They MUST pass on an airplane.

2. **Must run fast.** The whole sanity subfolder MUST complete in under
   30 seconds on a modern laptop. If a test takes longer, it belongs in
   a different category (`build`, `replay`, etc.).

3. **Must never be `#[ignore]`.** Sanity tests run unconditionally.
   Ignored tests live in other categories.

## What sanity tests cover

4. **Static validation** — file presence, label presence, count
   reconciliation (README claims vs. filesystem), README presence for
   every benchmark and agent.

5. **Dockerfile rule catalog** — the LABEL/pin/hygiene contract over every
   `benchmarks/*/Dockerfile`, `agents/*/Dockerfile`, and `models/*/Dockerfile`.
   Migrated to conftest/OPA (`policy/dockerfile/`, swept by
   `policy/dockerfile/run.sh`) for issue #114; the Rust `dockerfile_inspection`
   target keeps only the procedural unpinned-pip/npm walk that is a poor Rego
   fit. Each Rego rule still carries an ID, a severity (deny=Red / warn=Yellow),
   and a unit test.

6. **Trajectory rule catalog** — data-driven rules applied to every
   fixture under `tests/run/replay/fixtures/*.traces.jsonl` as OTLP/JSON.
   Same shape as the Dockerfile catalog.

7. **Compose contract** — every `benchmarks/*/compose.yaml` MUST schema-validate
   (the `check-compose-spec` pre-commit hook) and satisfy the eval markers + the
   `EVAL_*_VERSION`-not-as-tag rule (use `EVAL_*_TAG` per [/RULES.md](/RULES.md)
   rule 9). Migrated to conftest/OPA (`policy/compose/`, swept by
   `compose.sweep.sh`) for issue #114.

8. **Shared entrypoint contract** — `core/process-compose/run` (the
   framework launcher) MUST reference `EVAL_BENCHMARK_VERSION`,
   `EVAL_AGENT_VERSION`, and write `/output/task/version.json` +
   `/output/agent/version.json`.

## Adding a new rule

9. **Prefer mechanical over procedural.** If a new rule can be checked
   by reading file text, add it to the appropriate data-driven catalog.
   Don't write a new audit file.

10. **Rule IDs MUST match DOCKERFILE.md / TRAJECTORY.md / FLEET.md**
    where those still exist (they are being deprecated — see
    [../RULES.md](../RULES.md) rule 5). New rules skip the audit files
    entirely.

11. **Every new rule MUST ship with a unit test** that constructs a
    minimal bad example and asserts the rule fires, plus a minimal
    good example and asserts it does not.
