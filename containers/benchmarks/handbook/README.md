# HANDBOOK.md

**Status:** unreleased — files authored; build + end-to-end replay pending a Docker host (no `eval.benchmark.released` label yet; see [Release readiness](#release-readiness)).

**Paper:** [HANDBOOK.md](https://arxiv.org/abs/2607.25398) (Surge AI, 2026)
**Upstream:** [surge-ai/handbook](https://github.com/surge-ai/handbook) @ `86505865`

65 long-context agentic **instruction-following** tasks. An agent is dropped into a fake company — a file workspace plus 6 mock services (a core bash/python/PDF/file server + Gmail, Slack, Calendar, Jira, Shopify), **82 tools aggregated behind ONE streamable-HTTP MCP endpoint** — and must carry out a mundane request governed by a 20–124-page company handbook. Grading is **deterministic** (no LLM judge): per-task rubrics of Python `verify(workspace_path, external_services_path)` criteria of two kinds — `expected_output` (the change that must be present) and `incorrect_behavior` (out-of-scope state that must NOT change). Strict pass@1 = every criterion passes.

## Topology — proxy in the runner

Unlike enterpriseops-gym (third-party MCP images with embedded DBs → sidecars), HANDBOOK's servers are one uv workspace that natively co-locates the proxy with the workspace. So the MCP proxy runs **inside the runner** on `localhost:8000`, and grading runs locally in the same container. This is the simplest isolation ([RULES.md](../../../.agents/benchmarks/RULES.md) rule 6: file permissions over extra containers) and truest to upstream. A sidecar split is a documented future option if image size or isolation later demand it.

```
runner container
├── MCP proxy (localhost:8000/mcp)   ← started by /entrypoint.sh, 6 servers / 82 tools
│     ├─ syntara file server          runs as model uid 1000, operates in /workdir
│     └─ gmail/slack/calendar/jira/shopify   run as root
├── agent (uid 1002)                  ← reaches the proxy over localhost via its own tools
├── /workdir, /data                   chown model:model — servers read seeds, persist final.json
└── /root/tasks, /tests               root-only (700) — gold criteria, unreadable by agent AND model
```

## How the agent gets the endpoint

The endpoint is advertised as **prose inside `$TASK`** — the existing framework convention (as in enterpriseops-gym), NOT a first-class MCP channel. `setup_task.py` builds `$TASK` = the task's `system_prompt.md` (persona, date, tool families) + an `=== MCP SERVER ===` block (`url: http://localhost:8000/mcp`, `transport: streamable-http`) + the `instruction.md` user request. The agent receives only `TASK` and the model `*_BASE_URL`s (rule 7) and reaches the endpoint through its own shell/MCP tools.

> This relies on agents making MCP calls over their own tooling without first-class framework integration (per team review — Elron Bandel). Whether every agent does this reliably over 82 tools is exactly what the replay validation below measures.

## Grading

The MCP servers persist each service's state to `/data/<svc>/final.json` on **every write tool call** (upstream `@_snapshot_on_write`), so the gradable state is already on disk when the agent stops — no `export_state` call is needed. `/grade.sh` runs `sop_verifier.py` (root, post-agent), which:

1. materializes a compat external-services dir from `/data/<svc>/final.json` (falling back to the seed), remapping filenames to what the verifiers expect (`slack.json`→`slack_data.json`, `inbox.json`→`mailbox.json`, `calendar_data.json`→`calendar.json`, `jira_state.json`→`jira_data.json`);
2. execs each rubric's `verifier_code` over `/workdir` + that dir;
3. writes `reward = average rubric score` to `/logs/verifier/reward.txt` (rule 18) and a full `verifier_report.json` to `/output/task/` (per-rubric pass/fail + `passed` = strict pass@1).

## Running

**compose** (task via `EVAL_TASK_ID`, default 0):
```bash
EVAL_TASK_ID=0 EVAL_AGENT=openclaw EVAL_MODEL=anthropic/claude-opus-4-8 \
  docker compose -f containers/benchmarks/handbook/compose.yaml up --abort-on-container-exit
# → output/handbook/0/task/{result.json,verifier_report.json}
```

**k8s** (shared chart, no preset needed):
```bash
helm template containers/benchmarks/_chart \
  --set benchmark=handbook --set agent=openclaw --set task=0 | kubectl apply -f -
```

## Build

Shared-env benchmark built by the standard bake target (no `build.sh` — that path is only for per-task from-source builds, rule 24g). The one Dockerfile builds the MCP environment from the pinned upstream `docker/` context and stages all 65 tasks:

```bash
# benchmark image
eval-containers build bench --benchmark handbook
# or directly:
docker buildx bake -f containers/docker-bake.hcl benchmark-handbook

# combined eval image (benchmark + agent), canonical build args (rule 24a):
#   BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/handbook:latest
#   AGENT_IMAGE=ghcr.io/exgentic/agents/openclaw:latest
eval-containers build eval --benchmark handbook --agent openclaw
```

`ARG BENCHMARK_VERSION=86505865…` drives both the upstream fetch and the `eval.benchmark.data_revision` label (rule 4). The build clones the pinned revision (sparse: `docker/` then `tasks/`) and removes each clone within its layer, so the gold rubrics never persist in a readable layer.

## Faithfulness deviations

The eval-containers generic runner bounds the agent by `EVAL_TIMEOUT` wall-clock only. HANDBOOK's paper additionally caps runs at **200 tool calls** and **300s per call**; the generic runner does not enforce those per-call limits. Scores are therefore comparable on task outcome but not on the paper's exact interaction budget. `WORLDBENCH_CURRENT_TIME` (faketime) is left unset — matching upstream `task.toml`, which pins the date only in `system_prompt.md` prose.

## Release readiness

Blocked on two items (neither blocks running the benchmark):

1. **Replay fixture (rule 21a).** Needs one recorded end-to-end run (`tests/run/replay/fixtures/handbook-<task>-<agent>.traces.jsonl`) with `result.json` produced. Requires a Docker host — not available where these files were authored.
2. **Oracle `solution.sh` (rule 20a).** `add-benchmark` wants an oracle that derives gold = 1.0; HANDBOOK ships no per-task gold solution. Options: stay unreleased (still fully runnable) or build a replay-based oracle from a known-good trajectory. Needs a call.

Once a fixture lands and the replay sweep passes, add `LABEL eval.benchmark.released="true"` to the Dockerfile.

## Why `EVAL_AGENT` is meaningful here

HANDBOOK provides the environment, the handbook, and the request — never the agent. The value of this port is swapping in openclaw, Claude Code, Codex, OpenHands, or any other agent and getting a comparable deterministic score on the same arena.
