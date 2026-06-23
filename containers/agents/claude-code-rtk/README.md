# claude-code-rtk

Claude Code agent with [rtk (Rust Token Killer)](https://github.com/rtk-ai/rtk)
wired in as a context-compression layer.

## What it adds over `containers/agents/claude-code`

rtk is a single Rust binary that registers a `PreToolUse(Bash)` hook in Claude
Code's `settings.json`. Every Bash tool call the agent makes (`git diff`,
`pytest`, `cargo test`, `npm install`, ...) is rewritten to `rtk <cmd>` before
execution. rtk runs the command and emits compressed output, so the agent's
context window receives a fraction of the bytes it would otherwise.

Upstream claims 60–90 % token reduction on shell-heavy tasks
(`pytest -v`: 96 %, `cargo test`: 92 %, `git diff HEAD~1`: 94 %, `git status`:
80 %, `cat` source files: 95 %).

## Use

Identical UX to `claude-code`:

```bash
eval-containers run swe-bench --agent claude-code-rtk \
  --model openai/azure/gpt-5.4 --task-id sympy_1776_sympy-24066 --local
```

## Verification

End-to-end on **swe-bench Verified** task `sympy_1776_sympy-24066` (sympy
unit-system bug, 20-line patch, gateway: `gpt-5.4--bifrost`). Same task, same
model, same `EVAL_TIMEOUT`:

| | `claude-code` (baseline) | `claude-code-rtk` |
|---|---|---|
| Result | passed (reward 1) | passed (reward 1) |
| LLM calls | 31 | 31 |
| Bash tool calls | 5 | 7 |
| Input tokens (sum) | 1 023 840 | 1 038 019 |
| Output tokens (sum) | 33 578 | 35 388 |
| Wall time | ~9 min | ~9.5 min |

The hook fires correctly — verified during integration testing with a `tee`-based
PreToolUse sentinel writing each invocation's stdin to disk (45 fires recorded
across 7 Bash calls including 2 `git diff`, removed for production). On *this*
particular task the savings are within run-to-run noise: the agent only issued
a handful of Bash calls, and the patch was tiny so `git diff` output was
already small. **rtk's headline gains require voluminous Bash output**
(`pytest -v` with thousands of lines, big diffs, large `cat`s). A larger
benchmark sweep on test-loop-heavy tasks (django, astropy, etc.) is the right
way to quantify whether rtk pays off in expectation.

## Why a separate agent

- per-agent measurement: keeps a clean A/B in the fleet
- `claude-code` stays untouched, preserving its replay-fixture stability
- new agent gets its own labels (`eval.agent.rtk_version`) and bake target

## Versions

- `AGENT_VERSION` — Claude Code CLI pin (matches `claude-code`)
- `EVAL_RTK_VERSION` — rtk release pin (recorded in `eval.agent.rtk_version`).
  Build arg is `EVAL_RTK_VERSION`, not `RTK_VERSION`, because rtk's installer
  script reads `$RTK_VERSION` as a magic env var. The Dockerfile pipes
  `RTK_VERSION="${EVAL_RTK_VERSION#v}" sh` to the installer so the pinned
  version actually takes effect (the leading `v` is stripped because the
  installer expects `0.42.3`, not `v0.42.3`).

## Rules checked (`.agents/agents/RULES.md`)

| # | Rule | Compliance |
|---|---|---|
| 1 | Two scripts (`/opt/agent/install.sh`, `/run.sh`) | ✓ |
| 2 | Input via `$TASK` | ✓ inherited from claude-code |
| 3 | Output to stdout | ✓ inherited |
| 4 | Benchmark-agnostic | ✓ no benchmark-specific code |
| 5 | One protocol, one URL (`ANTHROPIC_BASE_URL`) | ✓ inherited |
| 6 | No embedded credentials (`sk-proxy` placeholder) | ✓ inherited |
| 7 | Unprivileged | ✓ |
| 8 | Limited filesystem | ✓ |
| 9 | External `EVAL_TIMEOUT` | ✓ no internal timeout |
| 10 | No self-sandboxing | ✓ Docker is the sandbox |
| 11 | Install on any base | ✓ install.sh symlinks only |
| 12 | Reproducible default (no env vars) | ✓ `AGENT_VERSION` and `EVAL_RTK_VERSION` pinned + passed through to installer |
| 13 | Version is a build arg | ✓ both versions are ARGs |
| 14 | Required labels | ✓ plus `eval.agent.rtk_version` |
| 15 | Build-time integration | ✓ no benchmark file modification |
| 16 | Build test | ✓ via fleet bake |
| 17 | Replay test | ⚠️ no recorded fixture yet — follow-up; claude-code's existing fixture isn't reusable since rtk's hook injection changes the trajectory |
| 18 | Smoke test | ✓ registered in `tests/run/agents/test.rs` |
