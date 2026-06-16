# Replay test rules

The replay category runs **full evaluation pipelines against recorded
trajectories** with zero LLM cost. It's the backbone of continuous
verification: prove that the benchmark image, agent image, eval combo,
and verifier all agree on a fixed LLM response without touching a real
provider.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **No API keys.** Replay tests MUST NOT read `ANTHROPIC_API_KEY`,
   `OPENAI_API_KEY`, or any provider credential. The replay model is
   the only LLM backend. This is the parent rule 7 made concrete for
   this category.

2. **Replay model indistinguishability.** The eval container MUST NOT
   know it is talking to a replay. It opens `http://model:4000` and
   gets the same HTTP contract as the real LiteLLM proxy. If the agent
   changes its request shape, the replay MAY diverge — that divergence
   is a signal, not a bug (re-record the fixture if intentional).

3. **testcontainers compose.** Replay tests use
   `testcontainers::compose::DockerCompose` to start the stack and
   rely on `Drop` cleanup. Raw `docker compose up` is forbidden per
   parent rule 6.

## Fixture lifecycle

4. **Fixtures are immutable ground truth.** Files under
   `tests/run/replay/fixtures/*.traces.jsonl` are PRODUCED by release
   verification's live fleet sweep. Contributors MUST NOT hand-edit
   fixtures; the fixture is the record of what a specific
   (benchmark, task, agent, model) combination actually produced
   under a specific release tag.

4a. **No live secrets or internal endpoints in a fixture.** A
   `*.traces.jsonl` MUST NOT contain a live credential (API key, token,
   OAuth token, password) or an internal-only endpoint (e.g. a
   `*.vpc-int.res.ibm.com` gateway host). The live sweep (rule 4) captures the
   eval container's environment, so it MUST redact these at capture. The secret
   scanner is the enforced backstop: `.github/.gitleaks.toml` scans the fixture
   tree for credential shapes (it does NOT trust the tree wholesale) and
   suppresses only the benign one-way observability hashes (`user_api_key_hash`,
   `prompt_cache_key`) and the base image's public `GPG_KEY` fingerprint — a real
   `sk-`/`ya29.`/internal-host value fails the scan. This is rule 1 (no API keys)
   extended from the replay runtime to the recorded artifact. A wholesale
   trust-the-tree allowlist once blinded this scan and let a live HF token, a
   LiteLLM key, and Google OAuth tokens through.

5. **Filename convention.** `{benchmark}-{task-id}-{agent}.traces.jsonl`.
   One fixture per (benchmark, task, agent) combination, stored as native
   OTLP/JSON — OpenTelemetry `gen_ai` semconv spans, one
   `ExportTraceServiceRequest` per line (what an otelcol `file` exporter
   writes). The model is fixed per release and recorded in
   `fixtures/provenance.json`.

6. **Provenance record.** `fixtures/provenance.json` MUST record, for
   every fixture: the model name and version, the agent version, the
   benchmark data_revision, the timestamp of capture, and the release
   tag of the live sweep that produced it. An orphan fixture without
   a provenance entry is drift.

7. **Broken manifest.** `fixtures/broken.json` marks fixtures whose
   recorded run is known-bad (refusals, wrong answers, content filter
   hits, max-tokens truncation). Findings on these are REPORTED but
   do NOT fail the test. They are scheduled for re-recording in the
   next release verification cycle.

## Adding a new fixture

8. **Fixtures are added by release verification.** A contributor SHOULD
   NOT commit a new `*.traces.jsonl` manually. New fixtures land as
   part of the release-verification live sweep commit.

9. **Emergency fixture addition.** If a new benchmark is added to the
   released set between release cycles, its fixture MAY be captured
   out-of-band via `eval-containers run <bench> --agent <agent> --model <model>`,
   as long as the provenance record is updated in the same commit.

## Core image dependency

10. **Core images MUST be available.** Replay's `ensure_images()` MUST
    rebuild `core/entrypoint`, `core/test-exact-match`, `core/litellm`,
    and `models/replay` before any replay test runs. The build sweep's
    `ImageGuard::Drop` deletes them after a prior sweep, so replay
    cannot assume they exist.
