<!--
Thank you for adding a new agent. This checklist exists because
shipping an agent is not just "docker build works" — it's "this agent
can actually execute a task against the replay model and against the
live gateway and produce a sane trajectory". Fill in every checkbox.
Reject the PR if any evidence section is empty.
-->

## Agent: `<name>`

<!-- One paragraph: who built it, what SDK, what primary model family, what paradigm (single-shot, multi-step, tool-heavy, code editor, etc.) -->

### Upstream

| Field | Value |
|---|---|
| Name | `<upstream name>` |
| URL | `<github URL>` |
| Pinned version | `<semver, git tag, or npm version>` |
| License | `<SPDX or link>` |
| Paper | `<arxiv link or n/a>` |
| Primary SDK | Anthropic / OpenAI / both / custom |
| Paradigm | single-shot / multi-step / tool-heavy / file-editor / other |

### Required Dockerfile labels (agents/RULES.md 14)

- [ ] `LABEL dock.type="agent"`
- [ ] `LABEL dock.agent.name="<name>"` (matches directory name)
- [ ] `LABEL dock.agent.version="<pinned version>"` — NOT `latest`, NOT empty
- [ ] `LABEL dock.agent.description="<one line>"`
- [ ] `LABEL dock.agent.runtime="node"` / `"python"` / `"go"` / whatever
- [ ] `LABEL dock.agent.url="<upstream>"`

### Required ENV (RULES.md principle 9)

- [ ] `ENV DOCK_AGENT_VERSION_DEFAULT="<same as dock.agent.version>"` declared after the LABEL block

### Image layout

- [ ] `/opt/agent/install.sh` installs the agent CLI exactly once. Used by the combination layer to re-install on top of a benchmark base without rebuilding the agent image from scratch.
- [ ] `/opt/agent/entrypoint.sh` is the agent's command. Reads `TASK` env var (mandatory, per agents/RULES.md 2), calls the CLI with `--dangerously-skip-permissions` / `--print` / whatever the agent's non-interactive flag is, pipes the answer to stdout.
- [ ] `/opt/agent/entrypoint.sh` routes LLM calls through `ANTHROPIC_BASE_URL` (for Anthropic SDK) or `OPENAI_BASE_URL` (for OpenAI SDK) — NOT a direct-to-provider URL. The eval container only talks to `http://model:4000`.
- [ ] `ENTRYPOINT ["/opt/agent/entrypoint.sh"]` (the standalone agent image works out of the box)

### Version override hook (optional but recommended)

- [ ] `/dock-reinstall-agent` script exists and re-installs the agent at the version given as its first arg. Invoked by `core/entrypoint/dock-entrypoint.sh` when `DOCK_AGENT_VERSION` differs from the baked default. If omitted, the agent refuses to run with an override (fail-loud). See `agents/claude-code/Dockerfile` for the reference implementation.

### Local build

- [ ] `dock build agent <name>` succeeds on your machine
- [ ] `cargo test --test dockerfile_inspection` passes with zero new red findings
- [ ] Image size ≤ 1 GB (or documented justification)

### Evidence: a real run against at least 2 benchmarks

<!--
Run the agent end-to-end against at least 2 benchmarks with distinct
characteristics (e.g. one MCQ + one code, or one short + one tool-using).
This catches agents that work on trivial prompts but fail on anything
that needs the full surface area.
-->

Benchmark 1 (recommend `aime` for math reasoning):

```bash
dock build eval aime --agent <name>
dock run aime --agent <name> --model gpt-5.4 --task-id 0 --local --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty, has real LLM calls
- [ ] `output/aime/0/task/result.json` has a valid reward (0, 1, or fractional)
- [ ] `output/aime/0/model/result.json` has `cost_usd > 0` — the proxy logged the call (if 0, investigate: your agent's SDK path may not trigger the logging callback, see `core/litellm/dock_logger.py`)

Benchmark 2 (recommend `humaneval` for code generation or `gsm8k` for tool-less reasoning):

```bash
dock build eval <second-benchmark> --agent <name>
dock run <second-benchmark> --agent <name> --model gpt-5.4 --task-id 0 --local --max-budget 1
```

- [ ] Same three checks as above
- [ ] Trace inspected using the [tests/live/RULES.md trace inspection checklist](../../tests/live/RULES.md#trace-inspection-checklist) (categories A-E). Verdict per benchmark:
  - Benchmark 1 — A/B/C/D/E: <!-- green / yellow / red per category + notes -->
  - Benchmark 2 — A/B/C/D/E: <!-- green / yellow / red per category + notes -->

### Replay fixture

- [ ] One of the two runs above (the greenest one) is promoted to `tests/replay/fixtures/<benchmark>-0-<name>.trajectory.jsonl`
- [ ] `fixtures/provenance.json` records the model, timestamp, and release tag
- [ ] `cargo test --test replay -- --ignored` passes including your new fixture

### Model compatibility matrix (if your agent doesn't use the standard endpoints)

Our proxy exposes `/v1/messages` (Anthropic) and `/v1/chat/completions` (OpenAI) and `/v1/responses` (OpenAI Responses API, LiteLLM v1.63.8+).

- [ ] My agent uses `/v1/messages` (Anthropic SDK)
- [ ] My agent uses `/v1/chat/completions` (OpenAI Chat Completions)
- [ ] My agent uses `/v1/responses` (OpenAI Responses API) — requires `core/litellm` pinned to v1.63.8+
- [ ] My agent uses a different endpoint (specify): <!-- ... --> — document which LiteLLM version is required

### Known limitations

<!-- "Agent hardcodes gpt-4o-mini as its default model, so you must
have that route in models/*/config.yaml" is a real limitation.
"Agent requires a GUI to install" is a limitation. Be specific. -->

### RULES.md changelog

- [ ] No RULES.md changes needed
- [ ] `agents/RULES.md` updated with a changelog entry dated today
