# Sanity test rules

The sanity category holds **fast mechanical gates** that run on every
plain `cargo test` — no `--ignored` flag, no container daemon, no
network. These are the gates a contributor's local build and every CI
job depend on for green/red signal in seconds.

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

5. **Dockerfile rule catalog** — data-driven rules applied to every
   `benchmarks/*/Dockerfile`, `agents/*/Dockerfile`, and
   `models/*/Dockerfile` as raw text. Every rule has an ID, a severity
   (Red / Yellow), a why, and a predicate `fn(&str, &str) -> bool`.

6. **Trajectory rule catalog** — data-driven rules applied to every
   fixture under `tests/replay/fixtures/*.trajectory.jsonl` as JSONL.
   Same shape as the Dockerfile catalog.

7. **Compose parse** — every `benchmarks/*/compose.yaml` MUST parse
   via `docker compose config` (static YAML parse, no daemon write).
   Plus: no compose `image:` field MUST use `EVAL_*_VERSION` as a
   placeholder (use `EVAL_*_TAG` per [/RULES.md](/RULES.md) rule 9).

8. **Shared entrypoint contract** — `core/entrypoint/eval-entrypoint.sh`
   MUST reference `EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, and
   write `/output/task/version.json` + `/output/agent/version.json`.

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
