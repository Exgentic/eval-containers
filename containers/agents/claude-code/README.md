# claude-code

Anthropic Claude Code: Anthropic's official command-line coding agent.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [anthropics/claude-code](https://github.com/anthropics/claude-code) |
| Version | `2.1.329` |
| Install mechanism | npm (`@anthropic-ai/claude-code`) |
| Language runtime | Node.js 22 |

## What it does

Claude Code is Anthropic's terminal coding agent: it reads, edits, and runs code in the current working directory using Claude models. It talks to the Anthropic API directly; Eval Containers points it at the LiteLLM proxy via `ANTHROPIC_BASE_URL` so every call is routed through `http://model:4000`. Strength: strong tool use and file editing tuned for Claude.

## How Eval Containers runs it

The entrypoint sets `ANTHROPIC_BASE_URL` and `ANTHROPIC_API_KEY`, and runs Claude Code as the run's model (`MODEL`): a Claude model by its Anthropic ID, so Claude Code uses that model's own window, reply cap and effort levels, any other model by its name. Compose's neutral `eval-model`, or no `MODEL`, keeps Claude Code's default; a launch's `ANTHROPIC_MODEL` wins. Then it runs `claude -p --dangerously-skip-permissions -- "$TASK"`. `-p` is print mode — the final answer is written to stdout. `--dangerously-skip-permissions` is safe because Docker is the sandbox. Claude Code's experimental betas, and with them tool search, keep Claude Code's own default (on); a launch sets `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1` for an endpoint that does not support them.

When the launch sets `EVAL_MODEL_CONTEXT_WINDOW` ([agents rules 25–26](../../../.agents/agents/RULES.md)), the entrypoint passes it to Claude Code's own compaction (and, for a model Claude Code doesn't know, as `CLAUDE_CODE_MAX_CONTEXT_TOKENS`) and caps replies at min(`CLAUDE_CODE_MAX_OUTPUT_TOKENS`, or 32000 unset, window/4). A cap past 20000 comes off the compaction window (`CLAUDE_CODE_AUTO_COMPACT_WINDOW`, or under 100k `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), so the compaction request still fits. A window under 64,000 fails before the first call. The derivation is in the `/run.sh` comment in the [Dockerfile](Dockerfile). Unset, Claude Code assumes the window it knows for a Claude model and 1M for any other, so tell the window of any smaller model.

## Version

Pinned to `2.1.329` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
