---
name: New agent request
about: Propose a new agent to add to the fleet.
title: "agent: <name> — <one-line summary>"
labels: ["new-agent"]
---

## Agent: `<name>`

<!-- One paragraph: who built it, what SDK, what primary model family,
what paradigm (single-shot, multi-step, tool-heavy, code editor, etc.) -->

## Upstream

| Field | Value |
|---|---|
| Name | `<upstream name>` |
| URL | `<github URL>` |
| Pinned version | `<semver / git tag / npm version>` |
| License | `<SPDX or link>` |
| Paper | `<arxiv link or n/a>` |
| Primary SDK | Anthropic / OpenAI / both / custom |
| Endpoint used | `/v1/messages` / `/v1/chat/completions` / `/v1/responses` / other |
| Paradigm | single-shot / multi-step / tool-heavy / file-editor / other |

## Why this agent

<!-- What shape of agent is NOT already covered by the current 17?
Different SDK? Different paradigm? Different tool surface? Answer
specifically. -->

## Fit with existing rules

- [ ] Agent has a non-interactive mode (can be run headless with
      a single `TASK` env var input; see [.agents/agents/RULES.md](../../.agents/agents/RULES.md) rule 2)
- [ ] Agent routes LLM calls through `ANTHROPIC_BASE_URL` or
      `OPENAI_BASE_URL` (no hardcoded provider URL)
- [ ] Agent CLI is pinnable to a specific version — not a moving
      `latest`
- [ ] Agent can be installed from a single `install.sh` step
- [ ] Does NOT require a GUI or interactive terminal

## Endpoint compatibility

<!-- If this agent uses OpenAI's Responses API (`/v1/responses`),
`core/litellm` must be pinned to v1.63.8 or newer. Check the
current pin before opening the PR. -->

## Known obstacles

<!-- Hardcoded default model that doesn't exist on our proxy?
Requires network access the eval container doesn't have? Needs
special tool setup? Say so here. -->

## Who implements

- [ ] I will open the PR myself
- [ ] I'm requesting someone else implement it
- [ ] I'll help review
