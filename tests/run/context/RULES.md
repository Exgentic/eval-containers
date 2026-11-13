# Context-window test rules

The context category answers one question per supporting agent: **told the
model's window (`EVAL_MODEL_CONTEXT`, agents rules 25–26), does the agent stay
inside it?** The static checks (`tests/static/check.rs`, `*_fits_the_window_it_is_told`)
compute each agent's compaction point from its settings. This suite runs the
agent and checks what it actually sends.

Parent: [.agents/verification/RULES.md](../../../.agents/verification/RULES.md)

## Scope

1. **Two runs per agent.** Every agent in the suite has a *told* run
   (`EVAL_MODEL_CONTEXT` set to the mock's window) and a *not told* run (unset).
   The told run passes when the agent compacts at least once, no request is
   rejected, and the run ends with the mock's final answer. The not-told run is
   the control: it passes when a request is rejected before any compaction. That
   shows the mock enforces its window, so the told run's pass means the window
   is what fixed it.

2. **Agents covered:** claude-code and codex. An agent is added with one
   `context!` line in `test.rs`.

3. **`#[ignore]` by default.** Each run starts two containers. Like the agents
   suite, this is release-verification territory, not per-PR.

## The mock

4. **A standalone script, not `models/replay`.** `mock.py` (stdlib Python) serves
   Anthropic Messages and OpenAI Responses on :4000. It runs as a second container
   from the agent's own `evals/agents-smoke--<name>` image (agents-smoke is
   `FROM python:3.12-slim`), so the suite adds no image and no replay code. It
   makes no upstream call, so no credential is involved (parent rule 7).

5. **It enforces a window as vLLM does.** A request counts as its body's
   bytes / 4 tokens. One whose tokens plus `max_tokens` exceed the window gets
   vLLM's context-length error (HTTP 400). Every answer reports that count as
   usage, so the agent's own compaction check sees the same number.

6. **It drives the run.** While a request offers a shell tool, the mock asks for
   `seq 1 2000` (~2.8k tokens a turn). It answers the agent's own compaction
   prompt with a summary, and the first ordinary request after one with "OK". It
   stops after 80 tool turns (`gave-up`), so a run that never compacts fails
   rather than hangs.

7. **Assertions read the mock's log.** One `[mock] {...}` JSON line per request
   on stderr: `tool`, `text`, `compact`, `final`, `reject`, `gave-up`. The test
   prints the full log when it fails.
