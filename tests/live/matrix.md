# Live fleet sweep matrix

Model: `gpt-5.4`  ·  Budget cap: $1/run  ·  Timeout: 600s

Agent rotation: task[i] → AGENTS[i % 6] where AGENTS = ["claude-code", "codex", "aider", "goose", "openhands", "gemini-cli"].

This file is regenerated on every `cargo test --test live`. It is the authoritative plan the `live_fleet_sweep` test will execute.

## Matrix

| # | Benchmark | Tasks on disk | Tasks chosen | Agent rotation |
|---|---|---|---|---|
| 1 | `advbench` | 520 | 6 | 0→claude-code, 103→codex, 207→aider, 311→goose, 415→openhands, 519→gemini-cli |
| 2 | `agentbench` | 300 | 6 | 0→claude-code, 59→codex, 119→aider, 179→goose, 239→openhands, 299→gemini-cli |
| 3 | `agentcompany` | 175 | 6 | 0→claude-code, 34→codex, 69→aider, 104→goose, 139→openhands, 174→gemini-cli |
| 4 | `agentdojo` | 86 | 6 | 0→claude-code, 17→codex, 34→aider, 51→goose, 68→openhands, 85→gemini-cli |
| 5 | `agentharm` | 176 | 6 | 0→claude-code, 35→codex, 70→aider, 105→goose, 140→openhands, 175→gemini-cli |
| 6 | `ai2d` | 3088 | 6 | 0→claude-code, 617→codex, 1234→aider, 1852→goose, 2469→openhands, 3087→gemini-cli |
| 7 | `aider-polyglot` | 225 | 6 | 0→claude-code, 44→codex, 89→aider, 134→goose, 179→openhands, 224→gemini-cli |
| 8 | `aime` | 90 | 6 | 0→claude-code, 17→codex, 35→aider, 53→goose, 71→openhands, 89→gemini-cli |
| 9 | `alpaca-eval` | 805 | 6 | 0→claude-code, 160→codex, 321→aider, 482→goose, 643→openhands, 804→gemini-cli |
| 10 | `apps` | 5000 | 6 | 0→claude-code, 999→codex, 1999→aider, 2999→goose, 3999→openhands, 4999→gemini-cli |
| 11 | `appworld` | 732 | 6 | 0→claude-code, 146→codex, 292→aider, 438→goose, 584→openhands, 731→gemini-cli |
| 12 | `arc` | 1172 | 6 | 0→claude-code, 234→codex, 468→aider, 702→goose, 936→openhands, 1171→gemini-cli |
| 13 | `arc-agi` | 120 | 6 | 0→claude-code, 23→codex, 47→aider, 71→goose, 95→openhands, 119→gemini-cli |
| 14 | `arena-hard` | 500 | 6 | 0→claude-code, 99→codex, 199→aider, 299→goose, 399→openhands, 499→gemini-cli |
| 15 | `assistantbench` | 33 | 6 | 0→claude-code, 6→codex, 12→aider, 19→goose, 25→openhands, 32→gemini-cli |
| 16 | `bbh` | 6511 | 6 | 0→claude-code, 1302→codex, 2604→aider, 3906→goose, 5208→openhands, 6510→gemini-cli |
| 17 | `bfcl` | 2000 | 6 | 0→claude-code, 399→codex, 799→aider, 1199→goose, 1599→openhands, 1999→gemini-cli |
| 18 | `bigcodebench` | 1140 | 6 | 0→claude-code, 227→codex, 455→aider, 683→goose, 911→openhands, 1139→gemini-cli |
| 19 | `browsecomp` | 1266 | 6 | 0→claude-code, 253→codex, 506→aider, 759→goose, 1012→openhands, 1265→gemini-cli |
| 20 | `chartqa` | 2500 | 6 | 0→claude-code, 499→codex, 999→aider, 1499→goose, 1999→openhands, 2499→gemini-cli |
| 21 | `code-contests` | 165 | 6 | 0→claude-code, 32→codex, 65→aider, 98→goose, 131→openhands, 164→gemini-cli |
| 22 | `coderefine` | 6545 | 6 | 0→claude-code, 1308→codex, 2617→aider, 3926→goose, 5235→openhands, 6544→gemini-cli |
| 23 | `commonsenseqa` | 1221 | 6 | 0→claude-code, 244→codex, 488→aider, 732→goose, 976→openhands, 1220→gemini-cli |
| 24 | `compilebench` | per-task-build (15) | 6 | curl→claude-code, curl→codex, curl→aider, curl→goose, curl→openhands, curl→gemini-cli |
| 25 | `core-bench` | 45 | 6 | 0→claude-code, 8→codex, 17→aider, 26→goose, 35→openhands, 44→gemini-cli |
| 26 | `docvqa` | 5349 | 6 | 0→claude-code, 1069→codex, 2139→aider, 3208→goose, 4278→openhands, 5348→gemini-cli |
| 27 | `drop` | 9535 | 6 | 0→claude-code, 1906→codex, 3813→aider, 5720→goose, 7627→openhands, 9534→gemini-cli |
| 28 | `gdpval` | 220 | 6 | 0→claude-code, 43→codex, 87→aider, 131→goose, 175→openhands, 219→gemini-cli |
| 29 | `global-mmlu` | 589764 | 6 | 0→claude-code, 117952→codex, 235905→aider, 353857→goose, 471810→openhands, 589763→gemini-cli |
| 30 | `gpqa-diamond` | 198 | 6 | 0→claude-code, 39→codex, 78→aider, 118→goose, 157→openhands, 197→gemini-cli |
| 31 | `gsm8k` | 1319 | 6 | 0→claude-code, 263→codex, 527→aider, 790→goose, 1054→openhands, 1318→gemini-cli |
| 32 | `harmbench` | 400 | 6 | 0→claude-code, 79→codex, 159→aider, 239→goose, 319→openhands, 399→gemini-cli |
| 33 | `healthbench` | 5000 | 6 | 0→claude-code, 999→codex, 1999→aider, 2999→goose, 3999→openhands, 4999→gemini-cli |
| 34 | `hellaswag` | 10042 | 6 | 0→claude-code, 2008→codex, 4016→aider, 6024→goose, 8032→openhands, 10041→gemini-cli |
| 35 | `humaneval` | 164 | 6 | 0→claude-code, 32→codex, 65→aider, 97→goose, 130→openhands, 163→gemini-cli |
| 36 | `humanevalplus` | 164 | 6 | 0→claude-code, 32→codex, 65→aider, 97→goose, 130→openhands, 163→gemini-cli |
| 37 | `ifeval` | 541 | 6 | 0→claude-code, 108→codex, 216→aider, 324→goose, 432→openhands, 540→gemini-cli |
| 38 | `kumo` | 250 | 6 | 0→claude-code, 49→codex, 99→aider, 149→goose, 199→openhands, 249→gemini-cli |
| 39 | `legalbench` | 19000 | 6 | 0→claude-code, 3799→codex, 7599→aider, 11399→goose, 15199→openhands, 18999→gemini-cli |
| 40 | `livecodebench` | 880 | 6 | 0→claude-code, 175→codex, 351→aider, 527→goose, 703→openhands, 879→gemini-cli |
| 41 | `longbench` | 3750 | 6 | 0→claude-code, 749→codex, 1499→aider, 2249→goose, 2999→openhands, 3749→gemini-cli |
| 42 | `math` | 5000 | 6 | 0→claude-code, 999→codex, 1999→aider, 2999→goose, 3999→openhands, 4999→gemini-cli |
| 43 | `math-500` | 500 | 6 | 0→claude-code, 99→codex, 199→aider, 299→goose, 399→openhands, 499→gemini-cli |
| 44 | `mathvista` | 1000 | 6 | 0→claude-code, 199→codex, 399→aider, 599→goose, 799→openhands, 999→gemini-cli |
| 45 | `mbpp` | 500 | 6 | 0→claude-code, 99→codex, 199→aider, 299→goose, 399→openhands, 499→gemini-cli |
| 46 | `mbppplus` | 378 | 6 | 0→claude-code, 75→codex, 150→aider, 226→goose, 301→openhands, 377→gemini-cli |
| 47 | `medmcqa` | 4183 | 6 | 0→claude-code, 836→codex, 1672→aider, 2509→goose, 3345→openhands, 4182→gemini-cli |
| 48 | `medqa` | 1273 | 6 | 0→claude-code, 254→codex, 508→aider, 763→goose, 1017→openhands, 1272→gemini-cli |
| 49 | `mgsm` | 2750 | 6 | 0→claude-code, 549→codex, 1099→aider, 1649→goose, 2199→openhands, 2749→gemini-cli |
| 50 | `mind2web` | 1009 | 6 | 0→claude-code, 201→codex, 403→aider, 604→goose, 806→openhands, 1008→gemini-cli |
| 51 | `minif2f` | 244 | 6 | 0→claude-code, 48→codex, 97→aider, 145→goose, 194→openhands, 243→gemini-cli |
| 52 | `mmlu` | 14042 | 6 | 0→claude-code, 2808→codex, 5616→aider, 8424→goose, 11232→openhands, 14041→gemini-cli |
| 53 | `mmlu-pro` | 12032 | 6 | 0→claude-code, 2406→codex, 4812→aider, 7218→goose, 9624→openhands, 12031→gemini-cli |
| 54 | `mmmu` | 900 | 6 | 0→claude-code, 179→codex, 359→aider, 539→goose, 719→openhands, 899→gemini-cli |
| 55 | `mrcr` | 2400 | 6 | 0→claude-code, 479→codex, 959→aider, 1439→goose, 1919→openhands, 2399→gemini-cli |
| 56 | `naturalquestions` | 3610 | 6 | 0→claude-code, 721→codex, 1443→aider, 2165→goose, 2887→openhands, 3609→gemini-cli |
| 57 | `niah` | 63 | 6 | 0→claude-code, 12→codex, 24→aider, 37→goose, 49→openhands, 62→gemini-cli |
| 58 | `ocrbench` | 1000 | 6 | 0→claude-code, 199→codex, 399→aider, 599→goose, 799→openhands, 999→gemini-cli |
| 59 | `olympiad-bench` | 910 | 6 | 0→claude-code, 181→codex, 363→aider, 545→goose, 727→openhands, 909→gemini-cli |
| 60 | `openbookqa` | 500 | 6 | 0→claude-code, 99→codex, 199→aider, 299→goose, 399→openhands, 499→gemini-cli |
| 61 | `piqa` | 1838 | 6 | 0→claude-code, 367→codex, 734→aider, 1102→goose, 1469→openhands, 1837→gemini-cli |
| 62 | `pubmedqa` | 1000 | 6 | 0→claude-code, 199→codex, 399→aider, 599→goose, 799→openhands, 999→gemini-cli |
| 63 | `ruler` | 200 | 6 | 0→claude-code, 39→codex, 79→aider, 119→goose, 159→openhands, 199→gemini-cli |
| 64 | `scibench` | 692 | 6 | 0→claude-code, 138→codex, 276→aider, 414→goose, 552→openhands, 691→gemini-cli |
| 65 | `scicode` | 65 | 6 | 0→claude-code, 12→codex, 25→aider, 38→goose, 51→openhands, 64→gemini-cli |
| 66 | `simpleqa` | 4326 | 6 | 0→claude-code, 865→codex, 1730→aider, 2595→goose, 3460→openhands, 4325→gemini-cli |
| 67 | `socialiqa` | 1954 | 6 | 0→claude-code, 390→codex, 781→aider, 1171→goose, 1562→openhands, 1953→gemini-cli |
| 68 | `swe-bench` | per-task-build (500) | 6 | sympy__sympy-24066→claude-code, sympy__sympy-24066→codex, sympy__sympy-24066→aider, sympy__sympy-24066→goose, sympy__sympy-24066→openhands, sympy__sympy-24066→gemini-cli |
| 69 | `swe-gym` | 2438 | 6 | 0→claude-code, 487→codex, 974→aider, 1462→goose, 1949→openhands, 2437→gemini-cli |
| 70 | `tau-bench` | 165 | 6 | 0→claude-code, 32→codex, 65→aider, 98→goose, 131→openhands, 164→gemini-cli |
| 71 | `theoremqa` | 800 | 6 | 0→claude-code, 159→codex, 319→aider, 479→goose, 639→openhands, 799→gemini-cli |
| 72 | `triviaqa` | 17944 | 6 | 0→claude-code, 3588→codex, 7177→aider, 10765→goose, 14354→openhands, 17943→gemini-cli |
| 73 | `truthfulqa` | 817 | 6 | 0→claude-code, 163→codex, 326→aider, 489→goose, 652→openhands, 816→gemini-cli |
| 74 | `usaco` | 307 | 6 | 0→claude-code, 61→codex, 122→aider, 183→goose, 244→openhands, 306→gemini-cli |
| 75 | `visualwebarena` | 910 | 6 | 0→claude-code, 181→codex, 363→aider, 545→goose, 727→openhands, 909→gemini-cli |
| 76 | `webarena` | 812 | 6 | 0→claude-code, 162→codex, 324→aider, 486→goose, 648→openhands, 811→gemini-cli |
| 77 | `winogrande` | 1267 | 6 | 0→claude-code, 253→codex, 506→aider, 759→goose, 1012→openhands, 1266→gemini-cli |
| 78 | `wmdp` | 3668 | 6 | 0→claude-code, 733→codex, 1466→aider, 2200→goose, 2933→openhands, 3667→gemini-cli |
| 79 | `wmt` | 9600 | 6 | 0→claude-code, 1919→codex, 3839→aider, 5759→goose, 7679→openhands, 9599→gemini-cli |
| 80 | `writingbench` | 1000 | 6 | 0→claude-code, 199→codex, 399→aider, 599→goose, 799→openhands, 999→gemini-cli |
| 81 | `xcopa` | 5500 | 6 | 0→claude-code, 1099→codex, 2199→aider, 3299→goose, 4399→openhands, 5499→gemini-cli |
| 82 | `xnli` | 75150 | 6 | 0→claude-code, 15029→codex, 30059→aider, 45089→goose, 60119→openhands, 75149→gemini-cli |
| 83 | `xstory-cloze` | 16621 | 6 | 0→claude-code, 3324→codex, 6648→aider, 9972→goose, 13296→openhands, 16620→gemini-cli |

## Summary

- Benchmarks in scope: **83** (81 normal + 2 per-task-build)
- Total runs: **498**
- Excluded (known-broken): see [tests/build/known-broken.md](../build/known-broken.md)
- Per-run wall time: ~1–10 min depending on agent verbosity
- Per-run cost ceiling: $1.00
- Gross budget ceiling: $498.00
