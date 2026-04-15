---
name: New benchmark request
about: Propose a new benchmark to add to the fleet.
title: "benchmark: <name> — <one-line summary>"
labels: ["new-benchmark"]
---

## Benchmark: `<name>`

<!-- One paragraph: what this benchmark measures, who built it upstream, why adding it to the fleet matters. -->

## Upstream

| Field | Value |
|---|---|
| Name | `<upstream name>` |
| URL | `<github or huggingface URL>` |
| Pinned revision | `<git sha or dataset revision>` |
| License | `<SPDX or link>` |
| Paper | `<arxiv link or n/a>` |
| Task count | `<N>` |
| Evaluation mode | exact-match / code-execution / LLM-judge / external |

## Why this benchmark

<!-- What gap in the current fleet does it fill? Is it covering a
language family, a reasoning style, a tool-use pattern, a domain,
a difficulty tier that isn't already represented? Answer
specifically — "it's popular" is not a reason. -->

## Fit with existing rules

- [ ] Data pattern is compatible with the single-JSONL convention
      ([benchmarks/RULES.md](../../benchmarks/RULES.md) rule 22)
- [ ] Grader is either exact-match, code-execution, LLM-judge, or
      can be written as a short shell script with a clear contract
- [ ] Upstream license allows redistribution of task metadata inside
      a Docker image
- [ ] Does NOT require more than 2 GB image size

## Known obstacles

<!-- If the upstream is gated (HF_TOKEN), x86_64-only, requires a
Meta signup form, etc. — say so here so the implementer knows what
they're walking into. -->

## Who implements

- [ ] I will open the PR myself
- [ ] I'm requesting someone else implement it
- [ ] I'll help review

## Related

- [ ] Not related to any tracked issue
- [ ] Related to issue #…
