# Agents that cannot support MCP

Companion to `broken.md`. That file lists agents that can't reach an
LLM through our gateway; this one lists agents that can't reach a
benchmark's tools over MCP.

Listed agents are excluded from `MCP_AGENTS` in `tests/agents/mcp.rs`
and MUST NOT write `/opt/agent/MCP` (doctrine/agents rule 19), so the
framework refuses to pair them with an MCP-declaring benchmark instead
of running them tool-blind.

An entry here means "upstream cannot do this", not "we haven't wired it
yet". An agent whose CLI supports MCP but whose `/run.sh` isn't wired is
simply unfinished and belongs in neither list. Adding an entry MUST cite
the root cause and the smallest viable fix; removing one is the success
condition.

## Cannot support MCP

| Agent | Root cause | Smallest fix |
|---|---|---|
| `aider` | No MCP client at any version. 0 of 773 files in the pinned tree match `mcp`; the upstream PR adding it (#5539) sits in `mergeable_state: unstable` and `main` last moved 2026-05-22. | Wait for #5539 to land, or drive MCP tools through aider's `/run` shell hook via a bundled client shim. |
| `open-interpreter` | No MCP client in the pinned 0.4.3 wheel (0 hits). Upstream issue #1623 was closed as *not planned*, and the repo has since been rewritten in Rust on top of Codex — the Python line this agent pins is unmaintained. | Bundle a ~50-line MCP-to-shell shim and expose it as a tool, or repin the agent to the Rust rewrite (which inherits Codex's MCP support) — a rewrite of the agent image, not a wiring change. |
| `bob` | Its Gemini-CLI fork *does* accept `httpUrl` MCP config, but the agent is already unusable in this harness for a more basic reason: the bundled JS hardcodes `api.us-east.bob.ibm.com` with no base-URL override. It cannot boot against our mock, so there is nothing to attach tools to. See `broken.md`. | Same as `broken.md` — this entry is downstream of that one and should be removed together with it. |
| `plandex` | Zero MCP references upstream, and the agent can't run non-interactively at all (see `broken.md`). `plandex.ai` currently NXDOMAINs. | Same as `broken.md`. |

## Notes

- Agents whose upstream MCP support exists but is *silently* fragile are
  not listed here — they are exactly what `tests/agents/mcp.rs` is for.
  Known cases: `crush` swallows MCP init errors entirely, `continue-cli`
  marks a server `error` and proceeds, `cline` and `gemini-cli` skip a
  malformed entry without a diagnostic. `codex`'s `required = true` is
  the only native fail-loud switch in the fleet.

- Four agents have no usable MCP client of their own and reach the
  servers through a bundled bridge instead: `mini-swe-agent`,
  `swe-agent`, and `terminus-2` get `agents/<name>/mcp-bridge`, a
  stdlib-only client exposed as an `mcp` command (their tool surface is a
  shell, so a command *is* the config dialect); `ra-aid` gets the same
  bridge wrapped as a `--custom-tools` LangChain module, because its own
  `MultiServerMCPClient_Sync` calls APIs removed in
  langchain-mcp-adapters ≥0.1.0. For all four the bridge lists tools at
  startup, so the handshake still precedes inference. Wiring work, not
  upstream blockers — hence not listed above.
