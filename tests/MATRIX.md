# E2E Test Matrix

Every benchmark appears once. Every agent appears 2–3 times, evenly spread.
Each row is one E2E test with a recorded replay fixture.

| Benchmark | Agent | Scoring | Pattern |
|-----------|-------|---------|---------|
| aime | claude-code | exact-match | shared-env |
| gpqa-diamond | codex | exact-match | shared-env |
| simpleqa | goose | exact-match | shared-env |
| math-500 | aider | exact-match | shared-env |
| mgsm | terminus-2 | exact-match | shared-env |
| mmlu-pro | openhands | exact-match | shared-env |
| hle | claude-code | exact-match | shared-env |
| mrcr | codex | exact-match | shared-env |
| humaneval | gemini-cli | code-execution | shared-env |
| mbpp | copilot-cli | code-execution | shared-env |
| livecodebench | gemini-cli | code-execution | shared-env |
| usaco | gemini-cli | code-execution | shared-env |
| ifeval | openclaw | fractional | shared-env |
| browsecomp | mini-swe-agent | llm-as-judge | shared-env |
| healthbench | goose | llm-as-judge | shared-env |
| kumo | codex | external | shared-env |
| gdpval | bob | external (HF upload) | shared-env |
| bfcl | openhands | custom | shared-env |
| appworld | terminus-2 | custom | shared-env |
| arc-agi | openclaw | custom | shared-env |
| mmmu | copilot-cli | custom | shared-env |
| aider-polyglot | aider | custom | shared-env |
| swe-bench | bob | swebench-grading | per-task |
| compilebench | mini-swe-agent | custom | per-task |
| terminal-bench | openhands | upstream | per-task |
| webarena | mini-swe-agent | webarena-verified | sidecar |
| osworld | claude-code | custom | sidecar |
| gaia | goose | exact-match | shared-env |
| tau-bench | (pass-through) | tau-bench-eval | bridge |

## Agent coverage

| Agent | Count | Benchmarks |
|-------|-------|------------|
| claude-code | 3 | aime, hle, osworld |
| codex | 3 | gpqa-diamond, mrcr, kumo |
| gemini-cli | 3 | humaneval, livecodebench, usaco |
| goose | 3 | simpleqa, healthbench, gaia |
| bob | 2 | gdpval, swe-bench |
| openclaw | 2 | ifeval, arc-agi |
| copilot-cli | 2 | mbpp, mmmu |
| aider | 2 | math-500, aider-polyglot |
| terminus-2 | 2 | mgsm, appworld |
| openhands | 3 | mmlu-pro, bfcl, terminal-bench |
| mini-swe-agent | 3 | browsecomp, compilebench, webarena |
| (pass-through) | 1 | tau-bench |
