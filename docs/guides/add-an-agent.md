# Add an agent

*Guide · for contributors · the canonical procedure is [`doctrine/agents/add-agent/SKILL.md`](../../doctrine/agents/add-agent/SKILL.md).*

Adding an agent is governed by doctrine. This page is a map — follow the skill
and the rules it links.

1. **Read the rules** — [`doctrine/agents/RULES.md`](../../doctrine/agents/RULES.md).
2. **Follow the skill** — [`doctrine/agents/add-agent/SKILL.md`](../../doctrine/agents/add-agent/SKILL.md).
3. **Extend a base image** — every agent extends `core/agent-base-<runtime>`
   (node, python, go, universal) rather than re-inlining shared setup
   ([`doctrine/RULES.md`](../../doctrine/RULES.md) principle 11).
4. **Open the PR** using the
   [agent PR template](../../.github/PULL_REQUEST_TEMPLATE/agent.md).

The agent talks only to the gateway's OpenAI-shaped endpoint and never sees the
real provider key — see [Isolation & gateways](../concepts/isolation-and-gateways.md).
