<!--
Thank you for adding a new canonical model. Canonical models are the
"model of record" axis of every evaluation — changing the canonical
model invalidates every existing replay fixture for the affected
benchmarks. This checklist exists because shipping a model is not
just "the proxy starts" — it's "the proxy routes every agent's
request, tracks every token's cost, and exposes the right endpoints
for every SDK we ship." Fill every checkbox.

Reviewers: reject the PR if the evidence section is empty or if any
section is marked "mostly".
-->

## Model: `<name>`

<!-- One paragraph: who built the upstream model, what provider,
what tier, why it fits as a canonical option (price, speed, quality
sweet spot, context window, etc.) -->

### Upstream

| Field | Value |
|---|---|
| Name | `<upstream name>` |
| Provider | openai / anthropic / azure / aws / gcp / custom |
| Model string passed to LiteLLM | `openai/azure/<x>` / `anthropic/<x>` / ... |
| API base (if not the provider default) | `<url or n/a>` |
| Per-1M-token input price | `<$X.YY>` |
| Per-1M-token output price | `<$X.YY>` |
| Context window | `<tokens>` |
| Tool-calling support | yes / no / partial |
| Responses API support | yes / no / n/a |

### Required Dockerfile labels (.agents/models/RULES.md 14, 15)

- [ ] `LABEL eval.type="model"`
- [ ] `LABEL eval.model.name="<name>"` (matches directory name)
- [ ] `LABEL eval.model.provider="<provider>"`
- [ ] `LABEL eval.model.litellm_version="<pinned version>"` (matches the `core/litellm` base image pin)

### Required ENV

- [ ] `ENV EVAL_LITELLM_VERSION_DEFAULT="<pinned version>"` (same value as the label)

### `config.yaml` contract

- [ ] `FROM ghcr.io/exgentic/core/litellm:latest` (inherits the shared proxy + eval-logger + budget wrapper)
- [ ] `model_list` has a wildcard route (`model_name: "*"`) aliasing to `<provider>/<model>`
- [ ] `model_list` ALSO has explicit aliases for every common name agents might send (gpt-4o, gpt-4o-mini, gpt-4.1, gpt-4.1-mini, gpt-5, gpt-5-mini, claude-3-5-sonnet, claude-3-5-haiku, claude-sonnet-4-5, gemini-2.5-pro, o1, o1-mini). The wildcard alone is NOT reliable on the OpenAI `/v1/chat/completions` path — explicit aliases are the fix, see the `models/gpt-5.4/config.yaml` reference.
- [ ] Every alias uses a YAML anchor (`&gpt54` / `*gpt54`) so the `litellm_params` block is declared once
- [ ] `litellm_settings.callbacks: ["eval_logger.eval_logger_instance"]` — cost tracking on
- [ ] `api_key: "os.environ/<PROVIDER>_API_KEY"` — NO hardcoded keys (.agents/models/RULES.md 5)
- [ ] `api_base: "os.environ/<PROVIDER>_API_BASE"` if the provider uses a non-default endpoint

### Key management

- [ ] API key name documented in `.env.example` with `<PROVIDER>_API_KEY=sk-...`
- [ ] No API key leaks to labels, compose files, or the agent container (.agents/models/RULES.md 4)
- [ ] Rate limit and spend budget documented in the PR description (if provider enforces one)

### Local build

- [ ] `eval-containers build model <name>` succeeds
- [ ] `cargo test --test dockerfile_inspection` passes with zero new red findings
- [ ] Image size ≤ 300 MB (it's just a config on top of core/litellm)

### Evidence: a real run

<!--
A model image that builds but can't route a real request is useless.
The evidence below exercises BOTH the `/v1/messages` path (claude-code)
and the `/v1/chat/completions` path (aider) against at least one
benchmark to prove both SDK surfaces work.
-->

```bash
# Claude path (Anthropic /v1/messages)
eval-containers run aime --agent claude-code --model <name> --task-id 0 --local --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty
- [ ] `output/aime/0/model/result.json` `cost_usd > 0` — eval_logger captured the call
- [ ] No `response_cost: 0.0` throughout the trajectory (cost tracking works on this path)

```bash
# OpenAI path (/v1/chat/completions)
eval-containers run aime --agent aider --model <name> --task-id 0 --local --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty
- [ ] `output/aime/0/model/result.json` `cost_usd > 0`
- [ ] Alias rewriting works: the agent's inbound model name (gpt-4.1-mini or similar) is translated to `<provider>/<name>` on the outbound call — verify by inspecting a trajectory entry's `custom_llm_provider` field

### Responses API path (OpenAI reasoning models only)

If this model targets an OpenAI reasoning endpoint (o1, o1-mini, gpt-5-*), also run:

```bash
eval-containers run aime --agent codex --model <name> --task-id 0 --local --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty
- [ ] Known issue: cost tracking on the `/v1/responses` path may not populate `response_cost` until [core/litellm/eval_logger.py](../../containers/core/litellm/eval_logger.py) is fixed. Document this in the PR description if it applies.

### Budget enforcement

- [ ] Smoke-test the `EVAL_MODEL_MAX_BUDGET` cap by running with `--max-budget 0.001` and confirming the run fails loud with `BudgetExceededError` before exhausting the per-run timeout

### Fixture promotion

- [ ] Fixtures under `tests/run/replay/fixtures/` remain unchanged — adding a model does NOT invalidate existing fixtures, it just adds a new option to the canonical matrix. Changing the "model of record" is a separate RFC process.

### Known limitations

<!-- "Provider rate-limits at 60 req/min so the live sweep needs
--test-threads=1" is a real limitation. "Model refuses safety
benchmarks" is a limitation. Be specific. -->

### RULES.md changelog

- [ ] No RULES.md changes needed
- [ ] `.agents/models/RULES.md` updated with a changelog entry dated today

### Docs ([.agents/docs/RULES.md](../../.agents/docs/RULES.md))

- [ ] User-facing knowledge this change adds or alters is reachable from `docs/` — nothing a user needs lives only in source/commits/heads (rule 13, sufficiency)
- [ ] Affected `docs/` pages updated in this PR (rule 15) and compliant with the docs rules
- [ ] No docs changes needed (purely internal change)
