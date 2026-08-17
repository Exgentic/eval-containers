# Gateway invariant test rules

The gateways category pins the assumptions we have about each of the
three gateway flavors — bifrost, litellm, portkey. Adding or modifying
a gateway MUST be reflected here so the test suite tracks the contract.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Three flavors, three contracts.** This file tests
   `gateways/<flavor>/` + `models/<flavor>/`. Adding a fourth
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

6a. **Env-priced cost emission (bifrost)** — with `EVAL_INPUT_COST_PER_TOKEN`
   + `EVAL_OUTPUT_COST_PER_TOKEN` set (USD per token), the rendered config
   MUST carry exactly one global wildcard `governance.pricing_overrides`
   entry patching those prices, and a served request's span MUST carry a
   cost attribute (`gen_ai.usage.cost`) equal to tokens × prices — asserted
   creds-free against the recording mock's fixed usage. With neither set,
   `pricing_overrides` MUST NOT render (bifrost's catalog pricing governs,
   unshadowed). With exactly one set, the container MUST exit non-zero
   naming both vars (gateways/RULES.md rule 22: misconfiguration is loud).

7. **litellm trajectory + result extras** — the `otel` callback MUST emit
   `gen_ai.*` spans (rule 6) which the otelcol sidecar writes to
   `/output/traces.jsonl` — the native OTLP/JSON trace that `models/replay`
   replays and the inspection rules read. litellm MUST additionally write
   `/output/result.json` (aggregated cost) and `/output/trajectory.jsonl`
   (LiteLLM StandardLoggingPayload) via the `eval_logger` callback. Both are
   properties of the litellm image, which this suite boots directly; neither
   feeds a fixture any more — fixtures are the edge's own record
   ([verification/RULES.md](../../../.agents/verification/RULES.md) 10).

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

## Model routing + pinning (translation contract)

Tested by `translation.rs`, no credentials. 36 assertions grouped into **10 tests
— one gateway boot per (flavor, config)**: the mode is fixed at boot, only the
inbound protocol varies per request, so every case in a group reuses one
container. Every-PR smoke = the 2 `native_pin` groups (the production default:
`EVAL_MODEL` set, `EVAL_MODEL_API` unset → pin + native wire + tools survive); the
other 8 groups (`translate_*`, `passthrough`) are `#[ignore]`-gated and run at
release with `--ignored`. Each case sends a client model DIFFERING from
`EVAL_MODEL` and checks the forwarded request against demands 19–22. The matrix,
over `{bifrost, litellm}`:

- **native_pin** — `EVAL_MODEL` set, `EVAL_MODEL_API` unset (the prod default):
  pin the model, keep the inbound wire, server tool survives.
- **translate_\*** — `EVAL_MODEL_API` set: force that wire. Matched inbound keeps
  its tool; cross-protocol inbound (full off-diagonal of the 3 search tools × a
  different wire) is translated + pinned, tool **known-lossy** (demand 21).
- **passthrough** — `EVAL_MODEL` unset: client model forwarded unchanged.

19. **Two model knobs, neither parsed.** `EVAL_MODEL` is a bare, opaque handle
    forwarded verbatim (`aws/claude-opus-4-8`, `azure/gpt-5.4`, …); the optional
    `EVAL_MODEL_API` (`anthropic|openai|gemini`) names the target wire. The test
    sets them via env only — NO `HOST`, NO mounted config — proving both gateways
    bind and route out of the box (`.agents/gateways/RULES.md` rules 2, 2b).

20. **`EVAL_MODEL` pins; `EVAL_MODEL_API` picks the wire.**
    - **Native pin** (default; API unset) — model rewritten to `EVAL_MODEL`, wire
      kept. bifrost — a small Caddy stamps the inbound wire into `X-Eval-Wire`,
      and one governance rule per provider keys on it (`headers['x-eval-wire'] ==
      '<p>'`) to pin on that wire; litellm — three family wildcards rewrite the
      model to `EVAL_MODEL` on each native provider.
    - **Wire override** (API set) — force `EVAL_MODEL_API`. bifrost — one
      `cel="true"` rule targets that provider; litellm — a single `*` entry.
    - **Passthrough** (`EVAL_MODEL` unset) — client model forwarded unchanged.

21. **Server tools survive iff protocol matches.** A web-search server tool
    (Anthropic `web_search_2*`, OpenAI Responses `web_search_preview`, Google
    `google_search`/`googleSearch`) MUST survive a **matched-protocol** forward
    intact — this is the invariant that broke (an Anthropic `web_search_20250305`
    was mangled into a bare `web_search`, upstream rejected it with
    `_websearch_interception_converted_stream: Extra inputs are not permitted`).
    **Cross-protocol** translation of a server tool is a known upstream
    limitation: the cross-protocol search cells mark it `ToolExpect::KnownLossy`,
    asserting only routing + pin, and emit a `NOTE` if a future engine version
    starts preserving the tool (then tighten that cell to `Require`) — the same
    known-limitation pattern as the portkey no-spans test.

22. **Assert on the forwarded request, not the response** (extends rule 10).
    `mock_upstream.py` (stock `python:3.12-slim`, no bake target) records every
    forwarded request to `/output/requests.jsonl`; the test reads that "target
    output request" and checks target-wire path + pinned model + tool. No
    upstream creds, deterministic. The
    `static_gateway_render_substitutes_every_template_var` guard checks every
    `${...}` a template uses is substituted by its `start` script.

23. **The change is confined to the two gateway dirs.** bifrost fronts its binary
    with a small Caddy that stamps the inbound wire into a header (its CEL rules
    can't see the request path); litellm keeps its Caddy path-shim. Agents,
    runner, compose, and helm are untouched.
