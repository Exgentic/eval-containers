<!--
Thank you for adding a new benchmark. This checklist exists because
shipping a benchmark is not just "docker build works" — it's "this
image produces a trace we can read and trust as ground truth". Fill
in every checkbox. Do NOT hide a "no" behind a "mostly" — if a check
fails, document it under "Known limitations" at the bottom instead.

Reviewers: reject the PR if any checkbox is missing its verdict or
if the evidence section is empty.
-->

## Benchmark: `<name>`

<!-- One paragraph: what this benchmark measures, who built it upstream, why it matters. -->

### Upstream

| Field | Value |
|---|---|
| Name | `<upstream name>` |
| URL | `<github or huggingface URL>` |
| Pinned revision | `<git sha or dataset revision>` |
| License | `<SPDX or link>` |
| Paper | `<arxiv link or n/a>` |
| Task count | `<N>` |
| Evaluation mode | exact-match / code-execution / LLM-judge / external |

### Required Dockerfile labels (benchmarks/RULES.md 15, 21a, 21b)

- [ ] `LABEL dock.type="benchmark"`
- [ ] `LABEL dock.benchmark.name="<name>"` (matches directory name)
- [ ] `LABEL dock.benchmark.description="<one line>"`
- [ ] `LABEL dock.benchmark.tasks="<N>"`
- [ ] `LABEL dock.benchmark.env="shared-env"` or `"per-task"`
- [ ] `LABEL dock.benchmark.internet="false"` (or `true` with justification)
- [ ] `LABEL dock.benchmark.data_revision="<sha>"` (pinned — no `main`, no `latest`)
- [ ] `LABEL dock.benchmark.url="<upstream>"`
- [ ] `LABEL dock.benchmark.paper="<arxiv or n/a>"`
- [ ] `LABEL dock.benchmark.upstream_base=...` (only if FROM references a third-party `ghcr.io` or similar)
- [ ] `LABEL dock.benchmark.released="true"` (only if this benchmark is ready for release verification — if so, a replay fixture is also required; see below)

### Required ENV (RULES.md principle 9)

- [ ] `ENV DOCK_BENCHMARK_VERSION_DEFAULT="<same as data_revision>"` declared after the LABEL block

### Task data pattern (benchmarks/RULES.md 22)

- [ ] Uses the single-JSONL pattern: build time writes `/tasks/all.jsonl` (one JSON row per task) with `chmod 600`
- [ ] Runtime `/entrypoint.sh` calls `/dock-materialize-task` (copied from `core/entrypoint`) instead of inlining its own python block
- [ ] Per-task field names match what `test.sh` expects (typically `id`, `problem`, `answer`)
- [ ] If the benchmark ships binary per-task assets (images, code, etc.): assets are base64-encoded in the JSONL row and decoded at runtime — or placed in a shared read-only dir and only metadata is per-task
- [ ] If this is a per-task-build benchmark: `FROM` line interpolates `${DOCK_TASK_ID}` and `tests/build/test.rs::per_task_build_args` has a curated known-good task id entry

### Grading (`tests/test.sh`)

- [ ] `COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh` (or benchmark-specific test.sh with a justification)
- [ ] `test.sh` reads `/logs/verifier/reward.txt` and writes an integer 0, 1, or fraction. Externally graded benchmarks MAY write `-1`.
- [ ] `test.sh` does NOT leak `EXPECTED_ANSWER` back to the agent (it's unset during the agent phase by `dock-entrypoint.sh` and restored for test.sh)

### Entrypoint

- [ ] `/entrypoint.sh` exports a TASK template that cites the problem from `/tasks/$DOCK_TASK_ID/problem.txt`
- [ ] `/entrypoint.sh` sets `EXPECTED_ANSWER` from `/tasks/$DOCK_TASK_ID/answer.txt`
- [ ] `/entrypoint.sh` ends with `exec /dock-entrypoint.sh` (no custom agent phase — the shared entrypoint is mandatory, see benchmarks/RULES.md 12)

### Compose (`compose.yaml`)

- [ ] Extends `compose/services.yaml` services
- [ ] `image:` field uses `${DOCK_AGENT_TAG:-latest}` (NOT `${DOCK_AGENT_VERSION:-latest}` — RULES.md principle 9)
- [ ] `cargo test --test compose` passes

### Local build

- [ ] `dock build bench <name>` succeeds on your machine
- [ ] For per-task-build: `dock build bench <name> --task-id <known-good>` succeeds
- [ ] `cargo test --test dockerfile_inspection` passes with zero new red findings
- [ ] Image size ≤ 2 GB (or documented justification)

### Evidence: a real run

<!--
Run the benchmark end-to-end against a real model, attach the result.
This is THE gate that distinguishes "builds" from "works". Reviewers
reject PRs that skip this step.
-->

```bash
dock build eval <name> --agent claude-code
dock run <name> --agent claude-code --model gpt-5.4 --task-id 0 --local --max-budget 1
```

- [ ] `output/<name>/0/task/result.json` exists with a valid reward
- [ ] `output/<name>/0/model/trajectory.jsonl` is non-empty with real LLM calls
- [ ] `output/<name>/0/task/input/problem.txt` is populated (sanity: what the agent actually saw)
- [ ] Trace inspected using the [tests/live/RULES.md trace inspection checklist](../../tests/live/RULES.md#trace-inspection-checklist) (categories A-E). Verdict per category:
  - A. Task delivered: <!-- green / yellow / red + note -->
  - B. Model engaged: <!-- green / yellow / red + note -->
  - C. Environment understood: <!-- green / yellow / red + note -->
  - D. Answer well-formed: <!-- green / yellow / red + note -->
  - E. Infrastructure hygiene: <!-- green / yellow / red + note -->

### Released-to-CI checkbox (benchmarks/RULES.md 21a)

If you're setting `dock.benchmark.released="true"`, you MUST also:

- [ ] Ship a replay fixture at `tests/replay/fixtures/<name>-0-<agent>.trajectory.jsonl`
- [ ] Add a `provenance.json` entry recording the model, timestamp, and release tag
- [ ] `cargo test --test replay -- --ignored` passes including your new fixture

If you're NOT setting released=true, leave the label off and this section is N/A.

### Known limitations

<!-- Be honest. "The upstream dataset requires HF_TOKEN to download"
is a real limitation and belongs here. "One of the tasks crashes if
the agent uses more than 10k tokens" is a limitation. "I didn't run
it with aider yet" is a limitation. -->

### RULES.md changelog

- [ ] No RULES.md changes needed
- [ ] `benchmarks/RULES.md` updated with a changelog entry dated today
