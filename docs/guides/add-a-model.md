# Use (or build) a model

*Guide · for everyone · the canonical rules are [`.agents/models/RULES.md`](../../.agents/models/RULES.md).*

The model is a **runtime** axis, and **using one never requires a build**. Two
paths — the generic gateway is the default; pinned per-model images are an
opt-in. You only build when you *author* one.

## Generic gateway (default) — any model, zero build

The default gateway routes whatever `EVAL_MODEL=<provider>/<model>` you set to
that provider, so any [LiteLLM-supported model](https://docs.litellm.ai/docs/providers)
works with no per-model image:

```bash
echo "OPENAI_API_KEY=sk-..." > .env
eval-containers run aime --task-id 0 --agent codex --model openai/gpt-5.4
# or --model anthropic/claude-sonnet-4-5, gemini/gemini-2.5-pro, openai/azure/<deployment>, …
```

`EVAL_MODEL` must be `<provider>/<model>` form — the generic gateway errors on a
bare name or an empty value (no silent default). Pick the proxy backend with
`EVAL_GATEWAY_IMAGE` (default `bifrost`; `litellm` and `portkey` also ship).

## Pinned per-model image — a shared, custom artifact (still zero build)

A per-model image bakes one model plus its config (cost rates, endpoint,
params). It's a **named, versioned artifact** teams share — "run it against
`models/gpt-5.4`" gives everyone the exact same pinned setup, which is the point
for cross-team reproducibility and per-model customization. Use a published one
with no build and no `EVAL_MODEL` (the model is baked):

```bash
EVAL_GATEWAY_IMAGE=gpt-5.4 eval-containers run aime --task-id 0 --agent codex
```

The gateway holds the real provider key; the runner only ever sees the proxy —
see [Isolation & gateways](../concepts/isolation-and-gateways.md).

## Build a model image (the only build case)

You build + publish a `containers/models/<name>` image only to **author** one —
either a new pinned per-model artifact, or a new generic backend beside
`bifrost`/`litellm`/`portkey`. Then:

1. **Read the rules** — [`.agents/models/RULES.md`](../../.agents/models/RULES.md)
   (rule 1: the default is a generic runtime gateway; a pinned image is a
   deliberate option; rules 4–7: key isolation + tamper-proof logging).
2. **Honor the version axes** — pin a reproducible LiteLLM version, expose
   `EVAL_LITELLM_VERSION` (see [Environment variables](../reference/env-vars.md)).
3. **Open the PR** with the
   [model PR template](../../.github/PULL_REQUEST_TEMPLATE/model.md).
