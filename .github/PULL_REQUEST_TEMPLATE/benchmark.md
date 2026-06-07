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

### Required Dockerfile labels (doctrine/benchmarks/RULES.md 15, 21a, 21b)

- [ ] `LABEL eval.type="benchmark"`
- [ ] `LABEL eval.benchmark.name="<name>"` (matches directory name)
- [ ] `LABEL eval.benchmark.description="<one line>"`
- [ ] `LABEL eval.benchmark.tasks="<N>"`
- [ ] `LABEL eval.benchmark.env="shared-env"` or `"per-task"`
- [ ] `LABEL eval.benchmark.internet="false"` (or `true` with justification)
- [ ] `LABEL eval.benchmark.data_revision="<sha>"` (pinned — no `main`, no `latest`)
- [ ] `LABEL eval.benchmark.url="<upstream>"`
- [ ] `LABEL eval.benchmark.paper="<arxiv or n/a>"`
- [ ] `LABEL eval.benchmark.upstream_base=...` (only if FROM references a third-party `ghcr.io` or similar)
- [ ] `LABEL eval.benchmark.released="true"` (only if this benchmark is ready for release verification — if so, a replay fixture is also required; see below)

### Required ENV (RULES.md principle 9)

- [ ] `ENV EVAL_BENCHMARK_VERSION_DEFAULT="<same as data_revision>"` declared after the LABEL block

### Task data pattern (doctrine/benchmarks/RULES.md 22)

- [ ] Uses the single-JSONL pattern: build time writes `/tasks/all.jsonl` (one JSON row per task) with `chmod 600`
- [ ] Runtime `/entrypoint.sh` calls `/eval-materialize-task` (copied from `core/entrypoint`) instead of inlining its own python block
- [ ] Per-task field names match what `test.sh` expects (typically `id`, `problem`, `answer`)
- [ ] If the benchmark ships binary per-task assets (images, code, etc.): assets are base64-encoded in the JSONL row and decoded at runtime — or placed in a shared read-only dir and only metadata is per-task
- [ ] If this is a per-task-build benchmark: `FROM` line interpolates `${EVAL_TASK_ID}` and `tests/build/test.rs::per_task_build_args` has a curated known-good task id entry

### Grading (`tests/test.sh`)

- [ ] `COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /grade.sh` (or benchmark-specific test.sh with a justification)
- [ ] `grade.sh` reads task data and writes `/logs/verifier/reward.txt` as an integer 0, 1, or fraction. Externally graded benchmarks MAY write `-1`.
- [ ] `grade.sh` does NOT leak `EXPECTED_ANSWER` back to the agent (it's unset during the agent phase and restored for grading)
- [ ] **Every metric the benchmark reports lands in `task/result.json`**, with the primary metric named `reward` ([doctrine/compose/RULES.md](../../compose/RULES.md) rule 16). Additional metrics (e.g. `exact_match`, `f1`, `bleu`, `partial_credit`, `tool_calls`) are named fields alongside `reward`. `grade.sh` is the only writer; NO metric is left in stdout for downstream to parse.
- [ ] Paste the `task/result.json` from one real run here so a reviewer can see the exact field set:

<details><summary>Sample <code>task/result.json</code></summary>

```json
{
  "task_id": "0",
  "benchmark": "<name>",
  "reward": 1,
  "passed": true
  // + any benchmark-specific metric fields
}
```

</details>

### Entrypoint

- [ ] `/entrypoint.sh` exports a TASK template that cites the problem from `/tasks/$EVAL_TASK_ID/problem.txt`
- [ ] `/entrypoint.sh` sets `EXPECTED_ANSWER` from `/tasks/$EVAL_TASK_ID/answer.txt`
- [ ] `/entrypoint.sh` ends with `exec "$@"` (passes control to CMD — see doctrine/benchmarks/RULES.md 12)

### Compose (`compose.yaml`)

- [ ] Extends `compose/services.yaml` services
- [ ] `image:` field uses `${EVAL_AGENT_TAG:-latest}` (NOT `${EVAL_AGENT_VERSION:-latest}` — RULES.md principle 9)
- [ ] `cargo test --test compose` passes

### Local build

- [ ] `eval-containers build bench <name>` succeeds on your machine
- [ ] For per-task-build: `eval-containers build bench <name> --task-id <known-good>` succeeds
- [ ] `cargo test --test dockerfile_inspection` passes with zero new red findings
- [ ] Image size ≤ 2 GB (or documented justification)

### Evidence: a real run

<!--
Run the benchmark end-to-end against a real model, attach the result.
This is THE gate that distinguishes "builds" from "works". Reviewers
reject PRs that skip this step.
-->

```bash
eval-containers build eval <name> --agent claude-code
eval-containers run <name> --agent claude-code --model gpt-5.4 --task-id 0 --local --max-budget 1
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

### Released-to-CI checkbox (doctrine/benchmarks/RULES.md 21a)

If you're setting `eval.benchmark.released="true"`, you MUST also:

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
- [ ] `doctrine/benchmarks/RULES.md` updated with a changelog entry dated today

### Docs ([doctrine/docs/RULES.md](../../doctrine/docs/RULES.md))

- [ ] User-facing knowledge this change adds or alters is reachable from `docs/` — nothing a user needs lives only in source/commits/heads (rule 13, sufficiency)
- [ ] Affected `docs/` pages updated in this PR (rule 15) and compliant with the docs rules
- [ ] No docs changes needed (purely internal change)
