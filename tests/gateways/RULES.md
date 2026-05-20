# Gateway invariant test rules

The gateways category pins the assumptions we have about each of the
three gateway flavors — bifrost, litellm, portkey. Adding or modifying
a gateway MUST be reflected here so the test suite tracks the contract.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Three flavors, three contracts.** This file tests
   `gateways/<flavor>/` + `models/gpt-5.4--<flavor>/`. Adding a fourth
   flavor MUST extend the `FLAVORS` constant in `test.rs` and add the
   matching static/runtime invariants.

2. **Two execution buckets.** Tests split into:
   - Plain `#[test]` / `#[tokio::test]` — static checks + boot probes +
     no-credential protocol matrix. Run in every contribution
     verification.
   - `#[ignore]`-gated — upstream-credentialed calls + OTel emission
     verification. Run in release verification with `.env` populated.

3. **No silent skips.** A missing `OPENAI_API_KEY` for an `#[ignore]`
   test MUST panic with a clear message, not silently pass. Skipping is
   the user's job via the `--ignored` flag.

## What to assert

4. **Labels** — every flavor MUST declare:
   - `LABEL gateway.kind="<flavor>"` — matches the directory name
   - `LABEL gateway.<flavor>_version=...` — the pinned upstream version
   - `LABEL gateway.translates_protocols="true|false"` — accurate to
     the flavor's actual protocol coverage
   - For non-translating flavors: `LABEL gateway.protocols="<csv>"`
     enumerating what *does* work

5. **Protocol matrix** — for each (flavor, protocol) cell:
   - Translating flavors (bifrost, litellm): all three of /openai,
     /anthropic, /genai MUST return 200 with the protocol's native
     response shape (choices / content / candidates).
   - Non-translating flavors (portkey): unsupported protocols MUST
     return **501 Not Implemented** with a structured error body:
     `{"error": {"type": "not_implemented", "message": "..."}}`. The
     message MUST name the protocol and point at a working flavor.

6. **OTel emission** — every flavor that natively serves a protocol
   MUST emit OTel spans into `/output/traces.jsonl` containing the
   gen_ai semconv attributes:
   - `gen_ai.input.messages`
   - `gen_ai.output.messages`
   - `gen_ai.response.model`
   
   Wired via the standard `OTEL_EXPORTER_OTLP_ENDPOINT` env var (base
   URL — each flavor derives any provider-specific suffix internally).

7. **litellm trajectory extras** — litellm MUST additionally write
   `/output/trajectory.jsonl` (LiteLLM StandardLoggingPayload format,
   consumed by `models/replay`) and `/output/result.json` (aggregated
   cost). The `eval_logger` callback is the load-bearing dependency
   here — every replay fixture in the repo originated from it.

8. **Stripped-component regression guards** — when a component is
   removed from a flavor (e.g. the bifrost sidecar that portkey used to
   bundle), the test suite MUST grow a guard that fails loudly if
   anyone re-adds it. Forbidden patterns are listed by exact match in
   the static tests.

9. **Health probe coherence** — `gateways/<flavor>/health` MUST only
   probe components that actually run in the image. Probing a stripped
   sidecar is a no-symptom bug (gateway looks fine, healthcheck
   silently fails) so the suite asserts the absence of dead probes.

## What NOT to assert

10. **No upstream-specific assertions.** The test fires a request
    upstream and asserts on the *gateway's protocol output*, not on
    the LLM's content. "The model said exactly X" belongs in `live/`,
    not here.

11. **No per-task-id or per-benchmark coupling.** This category tests
    the gateways in isolation — no benchmark runner, no agent. Adding
    a benchmark or agent MUST NOT require updating this suite.

12. **No latency / cost SLOs.** Performance regressions belong in a
    separate benchmark category if they ever become a thing.

## Test container lifecycle

13. **All container work goes through testcontainers-rs** (parent
    rule 6). `GenericImage` for single-container tests,
    `GenericBuildableImage` for the build bootstrap, `Mount::bind_mount`
    for /output capture, and `.with_network(name)` for the otelcol +
    gateway pod pair. NO `Command::new("docker")` shell-outs.

14. **Images are built on first run.** `ensure_built()` in `test.rs`
    builds core/otel + every gateway flavor + every model wrapper into
    the local store via `tc_build_context`. Idempotent: subsequent
    test invocations hit the layer cache and add ~1 second.

15. **Networks are unique per pod.** OTel tests use
    `format!("gw-test-{flavor}-{nanos}")` for the bridge network so
    parallel `cargo test` threads don't collide on a shared name.

## Failure policy

16. **A failed static test blocks the PR.** Label drift or a re-added
    sidecar is fixable in seconds; the assertion exists to catch it
    before review.

17. **A failed runtime test blocks the PR.** Boot failures and 501
    drift on portkey are architectural regressions that the rest of
    the suite assumes hold.

18. **An `#[ignore]` failure blocks the release tag.** OTel emission
    breaks are silent on a unit-test pass but cascade into stale
    replay fixtures and broken observability for the next month — the
    release gate is the right place to enforce them.
