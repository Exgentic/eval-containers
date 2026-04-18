# vakra

VAKRA — a multi-hop, multi-source tool-calling benchmark (IBM Research).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 28 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [IBM/vakra](https://github.com/IBM/vakra) |
| Paper | [ibm-research/VAKRA dataset](https://huggingface.co/datasets/ibm-research/VAKRA) |
| Dataset revision | `1511b3a6ce0bb8df8aca2ae1b578510e150b6b7e` |

## What the agent sees

Each task is a compound query that can only be answered by chaining calls across multiple simulated tools (e.g., search, database, file-system, calculator). The prompt includes the tool catalog; the agent must orchestrate a correct sequence of calls to produce the final answer.

## How it's graded

Externally graded. The harness compares the agent's produced call sequence and final answer against the reference multi-hop trace, rewarding correct final answers and penalizing incorrect intermediate tool choices. Pass means the final answer matches ground truth with a well-formed tool-call trace.
