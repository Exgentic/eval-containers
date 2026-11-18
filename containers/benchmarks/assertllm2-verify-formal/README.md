# assertllm2-verify-formal

AssertLLM2-verify - judge RTL against its spec, with Yosys, SymbiYosys and Z3

[`assertllm2-verify`](../assertllm2-verify/)'s tasks, labels, answer and
grader, built on the same `core/assertllm2-harness` image, with these tools
added for the agent: Yosys 0.52 (with `yosys-abc` and `yosys-smtbmc`) and Z3 4.13.3 from Debian trixie, and SymbiYosys v0.52 from `core/sby`. The agent writes its own properties; none ship with the tasks. Bitwuzla is not included.

Every task was built with these tools, at these versions, before it became a
task (`check-tasks.py`). The tasks, how they were made, the label assumption
and how to read the scores: [`assertllm2-verify`](../assertllm2-verify/README.md).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 19 (3 designs: 3 golden, 14 individual mutants, 2 merged) |
| Environment | shared-env |
| Internet required | false |
| Resources | 4 CPU / 8G (compose and `_chart/presets/assertllm2-verify-formal.yaml`) |
| Released | no |
| Upstream | [https://github.com/hkust-zhiyao/AssertLLM2](https://github.com/hkust-zhiyao/AssertLLM2) |
| Dataset revision | `assertllm2-verify`'s |

## Run

```bash
TASK_ID=0 docker compose -f containers/benchmarks/assertllm2-verify-formal/compose.yaml up
eval-containers oracle assertllm2-verify-formal --local --task-id 0
```

Lean base: `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/assertllm2-verify-formal:latest`,
`AGENT_IMAGE=ghcr.io/exgentic/agents/<agent>:latest`, `AGENT_VERSION` per agent.
