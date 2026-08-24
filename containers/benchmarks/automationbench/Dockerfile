# AutomationBench (Zapier) — model-only, native-harness benchmark.
# 600 public tasks (6 business domains × 100): realistic SaaS workflows over
# ~47 *simulated* tools (Salesforce, Gmail, Slack, HubSpot, …). The agent makes
# tool calls that mutate an in-process WorldState; grading compares the final
# state to per-task assertions — deterministic, in-process, no external
# services and no LLM judge.
#
# We run AutomationBench's OWN harness (`auto-bench`) for one task, pointing its
# model endpoint at the fleet gateway. This is a model-only eval: the harness IS
# the agent scaffold (no user simulator, so — unlike tau-bench — no bridge /
# second gateway / pass-through agent). The runner (which has EVAL_TASK_ID)
# resolves the task, runs the harness, and writes the reward.
#
# Upstream: github.com/zapier/AutomationBench  (MIT)  pinned by commit.
# Requires Python >=3.13, so this image is FROM python:3.13-slim (the shared
# benchmark-base-external is 3.12) with the same apt/uv resilience stack.

# syntax=docker/dockerfile:1
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint

FROM python:3.13-slim

# Rule 4: one ARG drives both the install ref and the data_revision label, so
# the label can never disagree with the code baked in. No git tags exist
# upstream — pin by commit SHA.
ARG BENCHMARK_VERSION=4a8e1061254004d9dac807054eed33fad7d1ff14
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="automationbench"
LABEL eval.benchmark.description="AutomationBench (Zapier) - business-workflow agent tasks over simulated SaaS tools, state-verified"
LABEL eval.benchmark.tasks="600"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/zapier/AutomationBench"

# ── apt: git + jq (network-resilient, mirrors benchmark-base-external) ────
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing \
         && apt-get upgrade -y \
         && apt-get install -y --no-install-recommends \
           curl ca-certificates git jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

# ── AutomationBench + verifiers, pinned. `verifiers` is pinned to the version
# upstream's uv.lock resolves (0.2.1) so the assembled dataset — and therefore
# the sequential task index → task_name map below — is reproducible. Retry loop
# for transient PyPI flake.
ENV PIP_DISABLE_PIP_VERSION_CHECK=1
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries=5 --timeout=120 "verifiers==0.2.1" "git+https://github.com/zapier/AutomationBench.git@${BENCHMARK_VERSION}" \
      ; then ok=1; break; fi; \
      echo "pip attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "pip dependency install failed after 5 tries" >&2; exit 1; }

# ── Build the stable index → task_name map ───────────────────────────────
# The runner selects a single task by its sequential integer EVAL_TASK_ID, but
# the upstream CLI addresses tasks only by NAME (`--tasks <name>`). Assemble the
# 600 public tasks in upstream's canonical order (domain order × in-file order,
# no shuffle) and record one row per task as /tasks/all.jsonl. This mirrors
# upstream's own _task_name(): task_name lives in the (JSON-string) `info`, with
# a fallback to the top-level `task` key. Assert 600 so a dataset change is
# caught at build. Root-only (§5): the agent must not read the task store.
RUN python3 <<'PYEOF'
import json
from datasets import concatenate_datasets  # noqa: F401  (imported by upstream)
from automationbench.domains import get_combined_dataset, PUBLIC_DOMAINS

ds = get_combined_dataset(list(PUBLIC_DOMAINS))

def task_name(row):
    info = row.get("info")
    if isinstance(info, str):
        try:
            info = json.loads(info)
        except json.JSONDecodeError:
            info = {}
    if isinstance(info, dict) and info.get("task_name"):
        return info["task_name"]
    return row.get("task") or row.get("task_name")

names = [task_name(r) for r in ds]
assert len(names) == 600, f"expected 600 public tasks at pinned commit, got {len(names)}"
assert all(names), "every task must resolve to a non-empty task_name"

import os
os.makedirs("/tasks", exist_ok=True)
with open("/tasks/all.jsonl", "w") as out:
    for i, name in enumerate(names):
        out.write(json.dumps({"id": i, "task_name": name}) + "\n")
print(f"Wrote {len(names)} tasks to /tasks/all.jsonl")
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Shared task materializer (writes /tasks/$EVAL_TASK_ID/<field>.txt at runtime).
COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

WORKDIR /app
ENV EVAL_BENCHMARK=automationbench

# Single-task runner: resolve EVAL_TASK_ID → task_name, run the harness against
# the gateway, parse the export, and write the reward.
COPY run_automationbench.py /app/run_automationbench.py
RUN chmod 755 /app/run_automationbench.py

# The runner writes /logs/verifier/reward.txt itself; keep a fail-closed
# baseline grader so a run that never reaches the writer still scores 0.
RUN mkdir -p /logs/verifier && cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
