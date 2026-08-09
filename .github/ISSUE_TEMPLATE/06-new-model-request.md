---
name: Model won't route / needs a gateway backend
about: Most models need no issue — just set EVAL_MODEL. Open this only if that fails.
title: "model: <provider>/<model> — <one-line summary>"
labels: ["new-model"]
---

<!--
The model is a RUNTIME axis: set `EVAL_MODEL=<provider>/<model>` and put the
provider key in `.env` — any model works with no image, no build, no request.
There are no per-model images. See docs/guides/add-a-model.md.

Open this issue only if that DOESN'T work — e.g. it needs a new gateway
flavor beside bifrost / litellm / portkey.
-->

## Model: `<provider>/<model>`

<!-- What you ran (EVAL_MODEL=…, EVAL_GATEWAY_IMAGE=…) and what happened. -->

## Upstream

| Field | Value |
|---|---|
| Provider | openai / anthropic / azure / aws / gcp / custom |
| LiteLLM handle | `<provider>/<model>` |
| API base (if non-default) | `<url or n/a>` |
| Credentials env var | `<PROVIDER>_API_KEY` (+ `_API_BASE` if needed) |

## What's missing

- [ ] Needs a new gateway flavor (the existing bifrost / litellm / portkey can't reach it) — describe why
- [ ] It works already; requesting it be added to the docs / examples

## Who implements

- [ ] I'll open the PR
- [ ] Requesting someone else
- [ ] I'll help review
