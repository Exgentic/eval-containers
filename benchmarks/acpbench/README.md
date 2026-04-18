# acpbench

ACPBench — a benchmark for **Action, Change, and Planning** reasoning (IBM Research).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1040 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [ACPBench project page](https://ibm.github.io/ACPBench/) |
| Paper | [arXiv:2410.05669](https://arxiv.org/abs/2410.05669) |
| Dataset revision | `05e5883d9afbcfa3bdc8f270cb345ef0b9526d4a` |

## What the agent sees

Each task is a multiple-choice question about planning and reasoning over actions and state changes — "which action is applicable?", "what is the next state?", "is the goal reachable?". The prompt asks the agent to print only the single letter of the correct choice to stdout.

## How it's graded

Exact-match on the single-letter answer via `core/test-exact-match`. One task = one point.
