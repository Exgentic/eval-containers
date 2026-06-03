# Isolation & gateways

*Concept · for operators & contributors · derives from [`doctrine/RULES.md`](../../doctrine/RULES.md) (principle 5), [`doctrine/gateways/RULES.md`](../../doctrine/gateways/RULES.md).*

An evaluation is only trustworthy if the trajectory is honest. Eval Containers
guarantees that by keeping the model proxy out of the agent's reach.

## The boundary

```
   ┌─────────┐   sk-proxy    ┌──────────┐   real key   ┌──────────┐
   │ runner  │ ────────────▶ │ gateway  │ ───────────▶ │ provider │
   │ (agent) │   (logged)    │ (proxy)  │              │  (LLM)   │
   └─────────┘               └──────────┘              └──────────┘
```

- **Independent observation.** Every LLM call is logged by the gateway, not the
  agent. The agent must not know the proxy exists and must not be able to tamper
  with the recorded trajectory.
- **Secret isolation.** The runner holds only a stand-in proxy key (`sk-proxy`).
  The real provider key lives in the gateway. The agent never sees it.

This is why the trajectory the gateway records can be trusted: the thing being
evaluated cannot edit the record of what it did.

## How it shows up per mode

- **compose / container** — runner and gateway are separate units; the runner's
  environment carries `sk-proxy`, the gateway carries the real key.
- **job (k8s)** — same split, plus a `NetworkPolicy` on the runner so it can
  only reach the gateway. The real key comes from the cluster `eval-secrets`
  Secret (see [Deploy on Kubernetes](../guides/deploy-on-kubernetes.md)).

## Models are provider-agnostic

The gateway proxies through [LiteLLM](https://docs.litellm.ai/docs/providers),
so any supported provider (OpenAI, Anthropic, Google, Azure, Ollama, …) works
behind the same boundary. The agent always talks to one OpenAI-shaped endpoint;
routing happens in the gateway.
