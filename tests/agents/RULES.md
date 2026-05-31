# Agent smoke test rules

The agents category answers one question per agent: **does this agent
actually talk to an LLM when invoked?** It does NOT measure quality,
score reward, or check the answer — those are `live/`'s job. This
suite catches the dominant failure mode: an agent that crashes on
startup or can't reach the gateway.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **One test per agent.** Every entry in `AGENTS` MUST have a
   matching `agent_smoke!(name, "<agent>")` invocation. The
   compile-time assertion at the bottom of `test.rs` enforces this.

2. **The pass condition is "≥1 LLM call observed."** The mock LLM
   (models/replay) logs `[replay] 1/N: ...` on the first received
   request. The test polls the mock's stderr; the first match passes.
   Subsequent calls and the agent's final answer are out of scope.

3. **Mock, not upstream.** The suite uses `models/replay` with a
   one-line trivial fixture as the mock LLM. NO upstream calls are
   made — this MUST work offline.

4. **`#[ignore]` by default.** Each test starts two containers and
   waits up to `FIRST_CALL_TIMEOUT` for the first call (150s today).
   The full sweep is ~10 min; that's release-verification territory,
   not per-PR.

## What the mock does

5. **All three protocols, single fixture.** `tests/agents/fixture.jsonl`
   carries one OpenAI Chat-shaped response. The replay model
   canonicalizes it and re-emits as Anthropic Messages or Google
   Gemini if the inbound route demands. So the same fixture works
   regardless of which protocol the agent uses (per doctrine/agents/RULES.md
   rule 5, each agent reads one of the three base-URL env vars).

6. **Mock hostname is per-test.** The mock container's hostname is
   `mock-{agent}-{nanos}` (unique per test); the agent's base-URL
   envs all point at that hostname via bridge-network DNS. Networks
   are uniquely named so parallel test threads don't collide.

## What gets passed to the agent

7. **TASK="Reply with exactly: OK"** — short, deterministic, doesn't
   trigger lengthy reasoning chains. Agents that try to inspect a
   repo first (aider, swe-agent) may need a few extra seconds to
   bootstrap; the `FIRST_CALL_TIMEOUT` covers this.

8. **All three base URLs set.** Each agent reads exactly one
   (doctrine/agents/RULES.md rule 5), but we don't track per-agent which one,
   so all three are set to the mock. Bogus `sk-proxy` API keys —
   none reach upstream because the mock doesn't forward.

9. **No real credentials.** This suite MUST NOT reference real
   provider credentials. Adding a `OPENAI_API_KEY: ${...}` reference
   here is a contribution-verification rule violation (parent rule
   1.2).

## When tests fail

10. **Reported failure modes.** On timeout, the test prints both
    replay's stderr (so you can see if a request arrived but the
    marker was misread) and the agent's stdout+stderr (so you can
    see what the agent process actually did or what it crashed on).

11. **Known-broken agents.** An agent that legitimately can't make
    this work (e.g. requires a paid CLI license, depends on a
    runtime not in agent-base) graduates to `tests/agents/broken.md`
    with a citation — same convention as `tests/build/known-broken.md`
    + `tests/replay/broken.json`. The test stays in the file (don't
    silently delete) so the regression direction is "this got fixed."

## Prerequisites

12. **Agent images must be built.** This suite bootstraps only the
    mock LLM (models/replay). The 20 agent images are the
    `build` test's responsibility:

        cargo test --test build -- --ignored

    Listing 20 agent-base + per-agent images here would duplicate
    that bootstrap; the cost is paid once via `build` and amortizes
    across `replay`, `live`, and this suite.

## What NOT to assert

13. **No quality.** "The agent solved the task" → `live/`. Here we
    only check the agent reaches the LLM.

14. **No span shapes / telemetry.** OTel emission is in `gateways/`,
    not here. The mock isn't OTel-instrumented.

15. **No retry-loop behavior.** If the agent retries on a 4xx from
    the mock or loops indefinitely, the timeout catches it. We
    deliberately don't assert on call counts beyond "≥1."

## Lifecycle

16. **All container work via testcontainers-rs** (parent rule 6).
    `GenericImage` for both the mock and the agent;
    `Mount::bind_mount` for the fixture; `.with_network(name)` to
    couple them. NO `Command::new("docker")` shells.

17. **Cleanup is automatic.** `ContainerAsync` impls Drop; the
    network's last reference dropping triggers `docker network rm`
    via testcontainers internal state. A panicking test still
    cleans up because Drop runs during unwind.
