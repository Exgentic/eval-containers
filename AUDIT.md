# Project audit — the bottom line

Generated from `benchmarks/*/AUDIT.md` — do not hand-edit (regenerate with the
`audit-rollup` skill). `✓` verified · `✗` failing · `?` unchecked · `n/a` not applicable.
Safety is a rollup of the per-benchmark safety checks (`✓` only if all pass).
**Audited** is the date of the audit's commit (derived from it, not stored); `⚠`
means the benchmark changed after that commit, so the row is stale and needs a re-audit.

| Benchmark | Building | Running | Isolation | Oracle | Traces | Replicate | Safety | Audited |
|-----------|:--------:|:-------:|:---------:|:------:|:------:|:---------:|:------:|---------|
| acpbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| advbench | ? | ? | ? | ? | ? | ? | ? | — |
| agentbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| agentcompany | ? | ? | ? | ? | ? | ? | ? | — |
| agentdojo | ? | ? | ? | ? | ? | ? | ? | — |
| agentharm | ? | ? | ? | ? | ? | ? | ? | — |
| agents-smoke | ? | ? | ? | ? | ? | ? | ? | — |
| ai2d | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| aider-polyglot | ? | ? | ? | ? | ? | ? | ? | — |
| aime | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| alpaca-eval | ? | ? | ? | ? | ? | ? | ? | — |
| apps | ? | ? | ? | ? | ? | ? | ? | — |
| appworld | ? | ? | ? | ? | ? | ? | ? | — |
| arc-agi | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| arc | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| arena-hard | ? | ? | ? | ? | ? | ? | ? | — |
| assetopsbench | ? | ? | ? | ? | ? | ? | ? | — |
| assistantbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| bbh | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| bfcl | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| bigcodebench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| browsecomp | ? | ? | ? | ? | ? | ? | ? | — |
| chartqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| code-contests | ? | ? | ? | ? | ? | ? | ? | — |
| coderefine | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| commonsenseqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| compilebench | ? | ? | ? | ? | ? | ? | ? | — |
| core-bench | ? | ? | ? | ? | ? | ? | ? | — |
| cybench | ? | ? | ? | ? | ? | ? | ? | — |
| docvqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| drop | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| flores200 | ✗ | ? | ? | ? | ? | ? | ? | 2026-06-10 |
| frontiermath | ✗ | ? | ? | ? | ? | ? | ? | 2026-06-10 |
| gaia | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| gdpval | ? | ? | ? | ? | ? | ? | ? | — |
| global-mmlu | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| gpqa-diamond | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| gsm8k | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| harmbench | ? | ? | ? | ? | ? | ? | ? | — |
| healthbench | ? | ? | ? | ? | ? | ? | ? | — |
| hellaswag | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| hle | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| humaneval | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| humanevalplus | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| ifeval | ? | ? | ? | ? | ? | ? | ? | — |
| itbench | ? | ? | ? | ? | ? | ? | ? | — |
| kumo | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| legalbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| livecodebench | ? | ? | ? | ? | ? | ? | ? | — |
| longbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| math-500 | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| math | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mathvista | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mbpp | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mbppplus | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| medmcqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| medqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mgsm | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mind2web | ? | ? | ? | ? | ? | ? | ? | — |
| minif2f | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mle-bench | ? | ? | ? | ? | ? | ? | ? | — |
| mmlu-pro | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mmlu | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mmmu | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mrcr | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| mt-bench | ? | ? | ? | ? | ? | ? | ? | — |
| naturalquestions | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| niah | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| ocrbench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| olympiad-bench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| openbookqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| osworld | ? | ? | ? | ? | ? | ? | ? | — |
| piqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| pubmedqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| realworldqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| ruler | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| scibench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| scicode | ? | ? | ? | ? | ? | ? | ? | — |
| simpleqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| socialiqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| swe-bench-pro | ? | ? | ? | ? | ? | ? | ? | — |
| swe-bench | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| swe-gym | ? | ? | ? | ? | ? | ? | ? | — |
| swe-lancer | ? | ? | ? | ? | ? | ? | ? | — |
| tau-bench | ? | ? | ? | ? | ? | ? | ? | — |
| terminal-bench | ✓ | ? | ✓ | ✓ | ? | ? | ? | 2026-06-10 |
| theoremqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| triviaqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| truthfulqa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| usaco | ? | ? | ? | ? | ? | ? | ? | — |
| vakra | ? | ? | ? | ? | ? | ? | ? | — |
| visualwebarena | ? | ? | ? | ? | ? | ? | ? | — |
| webarena | ? | ? | ? | ? | ? | ? | ? | — |
| winogrande | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| wmdp | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| wmt | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| workarena | ? | ? | ? | ? | ? | ? | ? | — |
| writingbench | ? | ? | ? | ? | ? | ? | ? | — |
| xcopa | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| xnli | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |
| xstory-cloze | ✓ | ? | ? | ✓ | ? | ? | ? | 2026-06-10 |

**Totals:** building 62/101 · isolation 1/101 · oracle 62/101 · running 0/101 · traces 0/101 · replicate 0/101 · safety 0/101
