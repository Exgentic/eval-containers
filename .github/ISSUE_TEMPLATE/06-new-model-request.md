---
name: Model won't route / needs a gateway backend
about: Most models need no issue — just set EVAL_MODEL. Open this only if that fails.
title: "model: <provider>/<model> — <one-line summary>"
labels: ["new-model"]
---

<!--
The model is a RUNTIME axis: set `EVAL_MODEL=<provider>/<model>` and put the
provider key in `.env` — any LiteLLM-supported model works with no image, no
build, no request. See docs/guides/add-a-model.md.

Open this issue only if that DOESN'T work — e.g. the pinned LiteLLM version
doesn't support the provider yet, or it needs a new gateway backend beside
bifrost / litellm / portkey.
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

- [ ] LiteLLM doesn't support this provider in the pinned `core/litellm` version — link a version bump
- [ ] Needs a new gateway backend (the existing bifrost / litellm / portkey can't reach it) — describe why
- [ ] Want a **pinned per-model image** (a shared, custom-configured artifact teams run against via `EVAL_GATEWAY_IMAGE=<name>`) — not just runtime `EVAL_MODEL`
- [ ] It works already; requesting it be added to the docs / examples

## Who implements

- [ ] I'll open the PR
- [ ] Requesting someone else
- [ ] I'll help review
