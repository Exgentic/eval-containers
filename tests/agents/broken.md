# Known-broken agents for tests/agents/

This file documents agents that currently fail `tests/agents/test.rs`
with a specific, citable upstream-CLI or design-level constraint. Per
`tests/RULES.md` rule 11, listed agents are excluded from the macro
in `test.rs` until their entry is removed.

Adding an agent here MUST cite the root cause + the smallest viable
fix. Removing an agent here is the success condition — once the fix
lands, delete the entry and re-add `agent_smoke!(name, "agent")` in
`test.rs`.

## Currently broken

| Agent | Root cause | Smallest fix |
|---|---|---|
| `bob` | IBM-internal: bob's bundled JS hardcodes `api.us-east.bob.ibm.com` as the backend, the supported auth modes are all IBM-issued (`W3ID_SSO`, `LOGIN_WITH_GOOGLE`, `USE_BOBSHELL`, `COMPUTE_ADC`, `USE_VERTEX_AI`), and there is no `OPENAI_BASE_URL` / proxy override. Even with a valid `BOBSHELL_API_KEY` + `--auth-method api-key` the request still goes to IBM. IBM's own integration tests (`BOB_SHELL_CLI_INTEGRATION_TEST=true`) talk to the same backend — they just have access to it. | Network-level interception: DNS-rewrite `api.us-east.bob.ibm.com` to the mock IP via `/etc/hosts`, terminate TLS at a HTTPS-capable mock (self-signed cert), set `NODE_TLS_REJECT_UNAUTHORIZED=0`. Or: ship a real IBM key, accept that the test calls IBM. Neither is in the spirit of the smoke suite. |
| `plandex` | Self-hosted stack: plandex-server (Go binary) + Postgres + an internal LiteLLM proxy bundled into the agent image. The CLI is wired around an interactive TUI: `plandex new --full` prompts to "Connect your Claude subscription" before our `set-model default` can take effect, and `plandex tell` opens `/dev/tty` for keystroke handling. The documented `plandex models custom -f FILE --save` flow does register a custom provider, but the built-in model packs (`daily`, `oss`, etc.) still gate `tell` on their respective provider keys (OpenRouter, DeepSeek, …). Harbor's own wiki page confirms: *"there's no way to preset these settings on Harbor's end, so one must configure it manually"*. | Either: (a) build a fully non-interactive setup path upstream (plandex would have to ship a flag like `--non-interactive` that skips the connection-confirmation step and lets a custom-model pack be selected purely from the CLI), or (b) run plandex's full stack with a real upstream provider key (defeats the smoke test). Both are out of scope. |

## Out of scope for the test rules but worth noting

- The test does NOT test cloud portability (uid arbitrary-UID), only
  that the agent boots and makes an LLM call as the image-default uid.
  Cloud-portability validation is a separate experiment — see the
  Stage-1/2 manual checks we ran (uid 1002, --user 65534:0, full
  lockdown) against `agents/claude-code:non-root-test`.

- "Known-broken" is a structural property, not a quality judgment.
  Both `bob` and `plandex` work fine in production with their
  intended backends — they're not broken for users. They are broken
  for the *agent-smoke carrier path*, which assumes a generic LLM
  endpoint can be swapped in via `OPENAI_BASE_URL` /
  `ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL`. Bob and plandex
  are the two agents in the fleet that don't honor that contract.

- For plandex, the Dockerfile and entrypoint were rewritten so the
  image actually builds and runs (multi-stage `--platform=linux/amd64`
  fix, `/opt/agent/` layout so the combination image picks up all the
  files plandex-server needs, postgres-16 + the litellm-deps installed
  at combination-build-time, a fresh venv for python 3.12, plus the
  `litellm_proxy.py` ASGI module copied to a path that's importable).
  The blocker is purely the CLI's interactive flow — once plandex
  ships a non-interactive setup path, the agent should be one
  `agent_smoke!` invocation away from green.
