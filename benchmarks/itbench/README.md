# itbench

ITBench — IT-automation scenarios spanning SRE, CISO, and FinOps personas (IBM Research).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 10 (skeleton subset — full suite includes hundreds of scenarios) |
| Environment | shared-env |
| Internet required | false |
| Released | skeleton |
| Upstream | [itbench-hub/ITBench](https://github.com/itbench-hub/ITBench) |
| Paper | [Developing AI agents for IT automation tasks with ITBench](https://research.ibm.com/publications/developing-ai-agents-for-it-automation-tasks-with-itbench) |
| Dataset revision | `3c36485964b059621d439343009ec0e7b4d2354b` |

## What the agent sees

Each task is a realistic IT-operations scenario: a CISO incident-response question, an SRE troubleshooting runbook, or a FinOps cost-attribution problem. The agent is expected to produce a structured action plan or answer as JSON. The current image ships 10 CISO scenarios as a skeleton; the full benchmark includes a live Kubernetes/DB/web environment that Dock will layer in over subsequent releases.

## How it's graded

Externally graded. The agent's JSON output is compared against reference actions/answers via a benchmark-specific scoring script bundled in the image. The skeleton version reports pass/fail on exact-match against the reference; the full version will evaluate against a live environment.

## Status

Skeleton. Expected to expand as the upstream live-environment harness is integrated.
