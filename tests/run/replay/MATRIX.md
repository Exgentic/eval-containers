# E2E Test Matrix

> **Note:** representative snapshot from an earlier ~22-fixture plan, not the
> current set — the authoritative list is `tests/run/replay/fixtures/` plus the
> `replay_test!` entries in `test.rs`. The agent-coverage counts below may lag.

Every benchmark with a fixture appears once. Agents are spread across benchmarks.
Each row is one replay test with a recorded fixture.

## Shared-env benchmarks (22 fixtures)

| Benchmark | Agent | Scoring | Fixture |
|-----------|-------|---------|---------|
| aime | claude-code | exact-match | aime-0-claude-code |
| gpqa-diamond | opencode | exact-match | gpqa-diamond-118-opencode |
| simpleqa | goose | exact-match | simpleqa-0-goose |
| math-500 | aider | exact-match | math-500-0-aider |
| mgsm | codex | exact-match | mgsm-549-codex |
| mmlu-pro | openhands | exact-match | mmlu-pro-0-openhands |
| hle | claude-code | exact-match | hle-0-claude-code |
| mrcr | claude-code | exact-match | mrcr-0-claude-code |
| humaneval | claude-code | code-execution | humaneval-0-claude-code |
| mbpp | aider | code-execution | mbpp-299-aider |
| livecodebench | codex | code-execution | livecodebench-0-codex |
| usaco | codex | code-execution | usaco-0-codex |
| ifeval | claude-code | fractional | ifeval-0-claude-code |
| browsecomp | codex | llm-as-judge | browsecomp-0-codex |
| healthbench | claude-code | llm-as-judge | healthbench-0-claude-code |
| kumo | codex | external | kumo-0-codex |
| gdpval | claude-code | external (HF) | gdpval-0-claude-code |
| bfcl | codex | custom | bfcl-0-codex |
| appworld | claude-code | custom | appworld-0-claude-code |
| arc-agi | claude-code | custom | arc-agi-0-claude-code |
| mmmu | claude-code | custom | mmmu-0-claude-code |
| aider-polyglot | aider | custom | aider-polyglot-0-aider |
| gaia | goose | exact-match | gaia-0-goose |

## Per-task and sidecar benchmarks (TODO)

| Benchmark | Agent | Pattern | Status |
|-----------|-------|---------|--------|
| swe-bench | — | per-task | needs build-arg handling |
| compilebench | — | per-task | needs build-arg handling |
| terminal-bench | — | per-task (upstream) | needs Harbor image auth |
| webarena | — | sidecar | needs multi-sidecar support |
| osworld | — | sidecar | needs VM image (11GB) |
| tau-bench | — | bridge | needs two-model replay |

## Agent coverage

| Agent | Count | Benchmarks |
|-------|-------|------------|
| claude-code | 10 | aime, hle, mrcr, humaneval, mbpp, ifeval, healthbench, gdpval, arc-agi, mmmu |
| codex | 7 | gpqa-diamond, mgsm, livecodebench, usaco, browsecomp, kumo, bfcl |
| goose | 2 | simpleqa, gaia |
| aider | 2 | math-500, aider-polyglot |
| openhands | 1 | mmlu-pro |
| gemini-cli | 0 | (needs re-recording after fix) |
| copilot-cli | 0 | (needs re-recording after fix) |
| openclaw | 0 | (needs re-recording after fix) |
| bob | 0 | (untested) |
| terminus-2 | 0 | (untested) |
| mini-swe-agent | 0 | (untested) |
