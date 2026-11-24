# Hang test rules

The hang category answers one question per covered agent: **when a shell
command the model starts never returns, does the agent still end the task by
itself, before `EVAL_TIMEOUT`?** A task that hangs instead holds its pod until
the limit and is scored `timeout`, losing the model's work (#744).

Parent: [.agents/verification/RULES.md](../../../.agents/verification/RULES.md)

## Scope

1. **One run per agent.** The agent's `evals/agents-smoke--<name>` image runs
   against the mock with `EVAL_TIMEOUT=1200`. The run passes when the call's
   result reached the mock no sooner than the agent's bound and says how the
   agent ended the command, and the agent then exited 0 (`agent/.exit-code`)
   with the final answer on stdout. A result that comes back sooner (a refused
   or broken tool call) fails, so the bound is what the run proves. Exit 124
   (137 after the kill grace) means the agent hung until `EVAL_TIMEOUT`.

2. **Agents covered:** claude-code (its Bash tool backgrounds the command at
   120 s and stops it 10 min after the final answer) and pi (the image's 600 s
   default bash timeout). An agent is added with one `hang!` line in `test.rs`
   giving its bound and the text its result shows, once its image bounds such a
   command under `EVAL_TIMEOUT`.

3. **`#[ignore]` by default.** Each run starts two containers and takes 10–13
   minutes, the agents' own bounds. Like the agents suite, this is
   release-verification territory, not per-PR.

## The mock

4. **A standalone script, not `models/replay`.** `mock.py` (stdlib Python)
   serves Anthropic Messages and OpenAI Chat Completions on :4000. It runs as a
   second container from the agent's own `evals/agents-smoke--<name>` image
   (agents-smoke is `FROM python:3.12-slim`), so the suite adds no image and no
   replay code. It makes no upstream call, so no credential is involved (parent
   rule 7).

5. **It drives the run.** The first request that offers a shell tool is answered
   with a call to a Python sleep of ~11 days, with no timeout of its own (not
   `sleep N`, which Claude Code refuses before running). A request that carries
   that call's result is answered with `DONE`. Any other request gets `OK`.

6. **Assertions read the mock's log and the agent's output.** One
   `[mock] {...}` JSON line per request on stderr: `tool`, `final` (with the
   seconds since the call and the start of the tool's output) or `text`. The
   test prints the log, the agent's stdout and the tail of its stderr.
