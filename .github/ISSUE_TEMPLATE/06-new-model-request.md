---
name: New canonical model request
about: Propose a new canonical model to add to the fleet.
title: "model: <name> — <one-line summary>"
labels: ["new-model"]
---

## Model: `<name>`

<!-- One paragraph: provider, tier, why it's a good canonical choice.
Canonical = the model is a genuine option for the "model of record"
axis of an evaluation, not just a dev convenience. -->

## Upstream

| Field | Value |
|---|---|
| Provider | openai / anthropic / azure / aws / gcp / custom |
| Model string for LiteLLM | `<provider>/<model>` |
| API base (if non-default) | `<url or n/a>` |
| Input price per 1M tokens | `<$X.YY>` |
| Output price per 1M tokens | `<$X.YY>` |
| Context window | `<tokens>` |
| Tool calling | yes / no / partial |
| Supports Responses API (`/v1/responses`) | yes / no / n/a |

## Why this model

<!-- What gap does it fill? Cheapest-reliable, highest-quality,
fastest, largest-context, specific-domain? Be specific. -->

## Fit with existing rules

- [ ] API key can be loaded from `.env` via `<PROVIDER>_API_KEY`
      (no hardcoded keys in image config — [.agents/models/RULES.md](../../models/RULES.md) rule 5)
- [ ] LiteLLM already supports this provider in the current pinned
      version (`core/litellm` → `main-v<X>-stable`)
- [ ] Cost tracking populates `response_cost` on the endpoint paths
      this model uses (if unknown, this will be verified in the PR)
- [ ] No additional provider SDK install needed beyond what
      `core/litellm` already bundles

## Impact on existing fixtures

<!-- Adding a new model does NOT invalidate existing replay
fixtures — each fixture is tied to the model that produced it.
But if you're proposing this model as a REPLACEMENT for the
current model-of-record, that's a separate RFC: every released
benchmark's fixtures need re-recording. Say which case this is. -->

- [ ] Additive — new option, existing fixtures unchanged
- [ ] Replacement — proposes to change the model-of-record; linked
      to an RFC issue #…

## Who implements

- [ ] I will open the PR myself
- [ ] I'm requesting someone else implement it
- [ ] I'll help review
