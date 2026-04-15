# Live fleet sweep matrix

Model: `gpt-5.4`  ·  Budget cap: $1/run  ·  Timeout: 600s

Agent rotation: task[i] → AGENTS[i % 3] where AGENTS = ["claude-code", "codex", "aider"].

This file is regenerated on every `cargo test --test live`. It is the authoritative plan the `live_fleet_sweep` test will execute.

## Matrix

| # | Benchmark | Tasks on disk | Tasks chosen | Agent rotation |
|---|---|---|---|---|
| 1 | `advbench` | 520 | 3 | 0→claude-code, 260→codex, 519→aider |
| 2 | `agentbench` | 300 | 3 | 0→claude-code, 150→codex, 299→aider |
| 3 | `agentcompany` | 175 | 3 | 0→claude-code, 87→codex, 174→aider |
| 4 | `agentdojo` | 86 | 3 | 0→claude-code, 43→codex, 85→aider |
| 5 | `agentharm` | 176 | 3 | 0→claude-code, 88→codex, 175→aider |
| 6 | `ai2d` | 3088 | 3 | 0→claude-code, 1544→codex, 3087→aider |
| 7 | `aider-polyglot` | 225 | 3 | 0→claude-code, 112→codex, 224→aider |
| 8 | `aime` | 90 | 3 | 0→claude-code, 45→codex, 89→aider |
| 9 | `alpaca-eval` | 805 | 3 | 0→claude-code, 402→codex, 804→aider |
| 10 | `apps` | 5000 | 3 | 0→claude-code, 2500→codex, 4999→aider |
| 11 | `appworld` | 732 | 3 | 0→claude-code, 366→codex, 731→aider |
| 12 | `arc` | 1172 | 3 | 0→claude-code, 586→codex, 1171→aider |
| 13 | `arc-agi` | 120 | 3 | 0→claude-code, 60→codex, 119→aider |
| 14 | `arena-hard` | 500 | 3 | 0→claude-code, 250→codex, 499→aider |
| 15 | `assistantbench` | 33 | 3 | 0→claude-code, 16→codex, 32→aider |
| 16 | `bbh` | 6511 | 3 | 0→claude-code, 3255→codex, 6510→aider |
| 17 | `bfcl` | 2000 | 3 | 0→claude-code, 1000→codex, 1999→aider |
| 18 | `bigcodebench` | 1140 | 3 | 0→claude-code, 570→codex, 1139→aider |
| 19 | `browsecomp` | 1266 | 3 | 0→claude-code, 633→codex, 1265→aider |
| 20 | `chartqa` | 2500 | 3 | 0→claude-code, 1250→codex, 2499→aider |
| 21 | `code-contests` | 165 | 3 | 0→claude-code, 82→codex, 164→aider |
| 22 | `coderefine` | 6545 | 3 | 0→claude-code, 3272→codex, 6544→aider |
| 23 | `commonsenseqa` | 1221 | 3 | 0→claude-code, 610→codex, 1220→aider |
| 24 | `compilebench` | 15 | 3 | 0→claude-code, 7→codex, 14→aider |
| 25 | `core-bench` | 45 | 3 | 0→claude-code, 22→codex, 44→aider |
| 26 | `cybench` | per-task-build (40) | 3 | 0→claude-code, 0→codex, 0→aider |
| 27 | `docvqa` | 5349 | 3 | 0→claude-code, 2674→codex, 5348→aider |
| 28 | `drop` | 9535 | 3 | 0→claude-code, 4767→codex, 9534→aider |
| 29 | `gdpval` | 220 | 3 | 0→claude-code, 110→codex, 219→aider |
| 30 | `global-mmlu` | 589764 | 3 | 0→claude-code, 294882→codex, 589763→aider |
| 31 | `gpqa-diamond` | 198 | 3 | 0→claude-code, 99→codex, 197→aider |
| 32 | `gsm8k` | 1319 | 3 | 0→claude-code, 659→codex, 1318→aider |
| 33 | `harmbench` | 400 | 3 | 0→claude-code, 200→codex, 399→aider |
| 34 | `healthbench` | 5000 | 3 | 0→claude-code, 2500→codex, 4999→aider |
| 35 | `hellaswag` | 10042 | 3 | 0→claude-code, 5021→codex, 10041→aider |
| 36 | `humaneval` | 164 | 3 | 0→claude-code, 82→codex, 163→aider |
| 37 | `humanevalplus` | 164 | 3 | 0→claude-code, 82→codex, 163→aider |
| 38 | `ifeval` | 541 | 3 | 0→claude-code, 270→codex, 540→aider |
| 39 | `kumo` | 250 | 3 | 0→claude-code, 125→codex, 249→aider |
| 40 | `legalbench` | 19000 | 3 | 0→claude-code, 9500→codex, 18999→aider |
| 41 | `livecodebench` | 880 | 3 | 0→claude-code, 440→codex, 879→aider |
| 42 | `longbench` | 3750 | 3 | 0→claude-code, 1875→codex, 3749→aider |
| 43 | `math` | 5000 | 3 | 0→claude-code, 2500→codex, 4999→aider |
| 44 | `math-500` | 500 | 3 | 0→claude-code, 250→codex, 499→aider |
| 45 | `mathvista` | 1000 | 3 | 0→claude-code, 500→codex, 999→aider |
| 46 | `mbpp` | 500 | 3 | 0→claude-code, 250→codex, 499→aider |
| 47 | `mbppplus` | 378 | 3 | 0→claude-code, 189→codex, 377→aider |
| 48 | `medmcqa` | 4183 | 3 | 0→claude-code, 2091→codex, 4182→aider |
| 49 | `medqa` | 1273 | 3 | 0→claude-code, 636→codex, 1272→aider |
| 50 | `mgsm` | 2750 | 3 | 0→claude-code, 1375→codex, 2749→aider |
| 51 | `mind2web` | 1009 | 3 | 0→claude-code, 504→codex, 1008→aider |
| 52 | `minif2f` | 244 | 3 | 0→claude-code, 122→codex, 243→aider |
| 53 | `mle-bench` | per-task-build (75) | 3 | 0→claude-code, 0→codex, 0→aider |
| 54 | `mmlu` | 14042 | 3 | 0→claude-code, 7021→codex, 14041→aider |
| 55 | `mmlu-pro` | 12032 | 3 | 0→claude-code, 6016→codex, 12031→aider |
| 56 | `mmmu` | 900 | 3 | 0→claude-code, 450→codex, 899→aider |
| 57 | `mrcr` | 2400 | 3 | 0→claude-code, 1200→codex, 2399→aider |
| 58 | `naturalquestions` | 3610 | 3 | 0→claude-code, 1805→codex, 3609→aider |
| 59 | `niah` | 63 | 3 | 0→claude-code, 31→codex, 62→aider |
| 60 | `ocrbench` | 1000 | 3 | 0→claude-code, 500→codex, 999→aider |
| 61 | `olympiad-bench` | 910 | 3 | 0→claude-code, 455→codex, 909→aider |
| 62 | `openbookqa` | 500 | 3 | 0→claude-code, 250→codex, 499→aider |
| 63 | `piqa` | 1838 | 3 | 0→claude-code, 919→codex, 1837→aider |
| 64 | `pubmedqa` | 1000 | 3 | 0→claude-code, 500→codex, 999→aider |
| 65 | `ruler` | 200 | 3 | 0→claude-code, 100→codex, 199→aider |
| 66 | `scibench` | 692 | 3 | 0→claude-code, 346→codex, 691→aider |
| 67 | `scicode` | 65 | 3 | 0→claude-code, 32→codex, 64→aider |
| 68 | `simpleqa` | 4326 | 3 | 0→claude-code, 2163→codex, 4325→aider |
| 69 | `socialiqa` | 1954 | 3 | 0→claude-code, 977→codex, 1953→aider |
| 70 | `swe-bench` | per-task-build (500) | 3 | sympy__sympy-24066→claude-code, sympy__sympy-24066→codex, sympy__sympy-24066→aider |
| 71 | `swe-bench-pro` | per-task-build (731) | 3 | 0→claude-code, 0→codex, 0→aider |
| 72 | `swe-gym` | 2438 | 3 | 0→claude-code, 1219→codex, 2437→aider |
| 73 | `swe-lancer` | per-task-build (1488) | 3 | 0→claude-code, 0→codex, 0→aider |
| 74 | `tau-bench` | 165 | 3 | 0→claude-code, 82→codex, 164→aider |
| 75 | `terminal-bench` | per-task-build (89) | 3 | 0→claude-code, 0→codex, 0→aider |
| 76 | `theoremqa` | 800 | 3 | 0→claude-code, 400→codex, 799→aider |
| 77 | `triviaqa` | 17944 | 3 | 0→claude-code, 8972→codex, 17943→aider |
| 78 | `truthfulqa` | 817 | 3 | 0→claude-code, 408→codex, 816→aider |
| 79 | `usaco` | 307 | 3 | 0→claude-code, 153→codex, 306→aider |
| 80 | `visualwebarena` | 910 | 3 | 0→claude-code, 455→codex, 909→aider |
| 81 | `webarena` | 812 | 3 | 0→claude-code, 406→codex, 811→aider |
| 82 | `winogrande` | 1267 | 3 | 0→claude-code, 633→codex, 1266→aider |
| 83 | `wmdp` | 3668 | 3 | 0→claude-code, 1834→codex, 3667→aider |
| 84 | `wmt` | 9600 | 3 | 0→claude-code, 4800→codex, 9599→aider |
| 85 | `writingbench` | 1000 | 3 | 0→claude-code, 500→codex, 999→aider |
| 86 | `xcopa` | 5500 | 3 | 0→claude-code, 2750→codex, 5499→aider |
| 87 | `xnli` | 75150 | 3 | 0→claude-code, 37575→codex, 75149→aider |
| 88 | `xstory-cloze` | 16621 | 3 | 0→claude-code, 8310→codex, 16620→aider |

## Summary

- Benchmarks in scope: **88** (82 normal + 6 per-task-build)
- Total runs: **264**
- Excluded (known-broken): see [tests/build/known-broken.md](../build/known-broken.md)
- Per-run wall time: ~1–10 min depending on agent verbosity
- Per-run cost ceiling: $1.00
- Gross budget ceiling: $264.00
