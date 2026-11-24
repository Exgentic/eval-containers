# assertllm2-verify-nospec

AssertLLM2-verify - judge RTL with no spec, from the source alone

[`assertllm2-verify`](../assertllm2-verify/)'s tasks, labels, answer and
grader, built on the same `core/assertllm2-harness` image, with the
specification withheld: `/app` holds only `rtl/`, with no `spec.md` and no
figures (`ASSERTLLM2_SPEC=0`). The prompt says no specification is provided and
asks whether the RTL correctly implements the design it evidently intends or
contains a bug. As in `assertllm2-verify`, no simulator or formal tool is
installed.

It is the baseline for what the spec contributes: the same agent scored here
and on `assertllm2-verify` differs only in whether it read the spec.

**A lower bound, not a verification test.** Some mutants cannot be told from a
correct design without the spec. A reset value changed from 0 to 1, or a
constant nudged by one, is a legitimate design choice in isolation; only the
spec says which was meant. A mutant that breaks the design's own consistency
(a narrowed shift, a mismatched pack and unpack, a stuck handshake) can still
be found. Expect this variant to score below `assertllm2-verify` on mutants, and
read the gap as what the spec adds. The labels are the same assumed labels:
golden `correct`, every mutant `incorrect`.

The tasks, how they were made, the label assumption, grading and how to read
the scores: [`assertllm2-verify`](../assertllm2-verify/README.md).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 19 (3 designs: 3 golden, 14 individual mutants, 2 merged) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/hkust-zhiyao/AssertLLM2](https://github.com/hkust-zhiyao/AssertLLM2) |
| Dataset revision | `assertllm2-verify`'s |

## Run

```bash
TASK_ID=0 docker compose -f containers/benchmarks/assertllm2-verify-nospec/compose.yaml up
eval-containers oracle assertllm2-verify-nospec --local --task-id 7
```

Lean base: `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/assertllm2-verify-nospec:latest`,
`AGENT_IMAGE=ghcr.io/exgentic/agents/<agent>:latest`, `AGENT_VERSION` per agent.
