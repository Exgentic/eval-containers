# assertllm2-verify-sim

AssertLLM2-verify - judge RTL against its spec, with Icarus Verilog and Verilator

[`assertllm2-verify`](../assertllm2-verify/)'s tasks, labels, answer and
grader, built on the same `core/assertllm2-harness` image, with these tools
added for the agent: Icarus Verilog 12.0 (`iverilog`, `vvp`) and Verilator 5.032, with the `make` and `g++` its generated C++ is built with — Debian trixie's packages. The agent writes its own testbenches; none ship with the tasks.

Every task was built with these tools, at these versions, before it became a
task (`check-tasks.py`). The tasks, how they were made, the label assumption
and how to read the scores: [`assertllm2-verify`](../assertllm2-verify/README.md).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 19 (3 designs: 3 golden, 14 individual mutants, 2 merged) |
| Environment | shared-env |
| Internet required | false |
| Resources | 4 CPU / 8G (compose and `_chart/presets/assertllm2-verify-sim.yaml`) |
| Released | no |
| Upstream | [https://github.com/hkust-zhiyao/AssertLLM2](https://github.com/hkust-zhiyao/AssertLLM2) |
| Dataset revision | `assertllm2-verify`'s |

## Run

```bash
TASK_ID=0 docker compose -f containers/benchmarks/assertllm2-verify-sim/compose.yaml up
eval-containers oracle assertllm2-verify-sim --local --task-id 0
```

Lean base: `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/assertllm2-verify-sim:latest`,
`AGENT_IMAGE=ghcr.io/exgentic/agents/<agent>:latest`, `AGENT_VERSION` per agent.
