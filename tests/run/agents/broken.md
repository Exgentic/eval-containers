# Known-broken agents for tests/run/agents/

This file documents agents that currently fail `tests/run/agents/test.rs`
with a specific, citable upstream-CLI or design-level constraint. Per
`tests/run/agents/RULES.md` rule 11, listed agents are excluded from the
macro in `test.rs` until their entry is removed.

Adding an agent here MUST cite the root cause + the smallest viable
fix. Removing an agent here is the success condition — once the fix
lands, delete the entry and re-add `agent_smoke!(name, "agent")` in
`test.rs`.

## Currently broken

| Agent | Root cause | Smallest fix |
|---|---|---|
| `bob` | IBM-internal: bob's bundled JS hardcodes `api.us-east.bob.ibm.com` as the backend, the supported auth modes are all IBM-issued (`W3ID_SSO`, `LOGIN_WITH_GOOGLE`, `USE_BOBSHELL`, `COMPUTE_ADC`, `USE_VERTEX_AI`), and there is no `OPENAI_BASE_URL` / proxy override. Even with a valid `BOBSHELL_API_KEY` + `--auth-method api-key` the request still goes to IBM. IBM's own integration tests (`BOB_SHELL_CLI_INTEGRATION_TEST=true`) talk to the same backend — they just have access to it. | Network-level interception: DNS-rewrite `api.us-east.bob.ibm.com` to the mock IP via `/etc/hosts`, terminate TLS at a HTTPS-capable mock (self-signed cert), set `NODE_TLS_REJECT_UNAUTHORIZED=0`. Or: ship a real IBM key, accept that the test calls IBM. Neither is in the spirit of the smoke suite. |

## Out of scope for the test rules but worth noting

- The test does NOT test cloud portability (uid arbitrary-UID), only
  that the agent boots and makes an LLM call as the image-default uid.
  Cloud-portability validation is a separate experiment — see the
  Stage-1/2 manual checks we ran (uid 1002, --user 65534:0, full
  lockdown) against `agents/claude-code:non-root-test`.

- "Known-broken" is a structural property, not a quality judgment.
  `bob` works fine in production with its intended backend — it's not
  broken for users. It is broken for the *agent-smoke carrier path*,
  which assumes a generic LLM endpoint can be swapped in via
  `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL`.
  Bob is the one agent in the fleet that doesn't honor that contract.
