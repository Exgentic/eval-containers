# Add a model

*Guide · for contributors · the canonical rules are [`.agents/models/RULES.md`](../../.agents/models/RULES.md).*

A model image is a LiteLLM-backed gateway that proxies to an upstream provider
and logs every call. Adding one is governed by doctrine.

1. **Read the rules** — [`.agents/models/RULES.md`](../../.agents/models/RULES.md)
   (what a model image must be).
2. **Pick the provider** — any
   [LiteLLM-supported provider](https://docs.litellm.ai/docs/providers) works
   (OpenAI, Anthropic, Google, Azure, Ollama, …); routing lives in the gateway.
3. **Honor the version axes** — ship a reproducible default tag, and expose the
   internal LiteLLM version via `EVAL_LITELLM_VERSION`
   ([`.agents/RULES.md`](../../.agents/RULES.md) principle 9; see
   [Environment variables](../reference/env-vars.md)).
4. **Open the PR** using the
   [model PR template](../../.github/PULL_REQUEST_TEMPLATE/model.md).

The gateway holds the real provider key; the runner only ever sees the proxy —
see [Isolation & gateways](../concepts/isolation-and-gateways.md).
