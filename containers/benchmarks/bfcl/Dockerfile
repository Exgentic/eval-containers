# BFCL (Berkeley Function Calling Leaderboard) — model-only, native-harness.
#
# BFCL grades a function call by AST-checking the model's tool call against a
# list of acceptable argument values — NOT by string-matching printed text.
# The previous image flattened each task to a prompt and graded stdout with
# core/test-exact-match; that can never match BFCL's ground-truth schema, so
# every task scored 0 regardless of correctness. This image instead runs
# BFCL's OWN harness + AST checker (the thing that actually knows how to grade)
# for one task, pointing its OpenAI-compatible endpoint at the fleet gateway.
#
# This is a model-only eval, exactly like automationbench: the harness IS the
# agent scaffold (native function-calling, one model turn, no user simulator,
# so no bridge / second gateway / pass-through agent). The runner (which holds
# EVAL_TASK_ID) resolves the task, drives BFCL in-process, and writes the
# reward from BFCL's per-category score file.
#
# Scope: the six non-live, single-turn AST categories (simple_python,
# simple_java, simple_javascript, multiple, parallel, parallel_multiple) —
# 1150 tasks, deterministically gradable with no external services. Live,
# multi-turn, memory, and web-search categories are intentionally excluded
# (they need dataset/network state this image does not carry).
#
# Upstream: github.com/ShishirPatil/gorilla (Apache-2.0), pinned by commit;
# the SAME pin supplies the harness, the bundled BFCL_v4 data, and the AST
# checker, so they can never disagree. Core deps are API-only (openai, tree-
# sitter); vLLM/SGLang are optional extras we never install.

# syntax=docker/dockerfile:1
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint

FROM python:3.12-slim

# Rule 4: one ARG drives both the install ref and the data_revision label, so
# the label can never disagree with the code+data baked in. No usable release
# tags upstream for this subdir — pin by commit SHA.
ARG BENCHMARK_VERSION=6ea57973c7a6097fd7c5915698c54c17c5b1b6c8
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="bfcl"
# Released: a model-only replay fixture (bfcl-0-claude-code) was recorded from a
# live gateway run (azure/FW-GLM-5.2) and registered as a replay test. It
# replaces the removed legacy exact-match fixtures, which no longer described
# this benchmark after the grading paradigm changed to model-only (§21a).
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Berkeley Function Calling Leaderboard - native function-calling, AST-checked (non-live single-turn categories)"
LABEL eval.benchmark.tasks="1150"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/ShishirPatil/gorilla"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/bfcl"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bfcl"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.06840"

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

# ── BFCL, pinned. sentence-transformers (a core dep) pulls torch; install the
# CPU-only wheel first so the default CUDA build (~2 GB of unused GPU libs) is
# never fetched — this is an API-only eval. Then install bfcl_eval itself,
# which sees torch already satisfied. Retry loop for transient PyPI flake.
#
# soundfile is pinned explicitly: bfcl_eval's model registry imports qwen_agent
# at module load (constants.model_config → api_inference.qwen), and qwen_agent's
# utils import `soundfile` unconditionally — yet nothing in the dependency graph
# declares it, so without this the very first `import bfcl_eval...` in the runner
# fails with ModuleNotFoundError. The wheel bundles libsndfile (no apt package
# needed). We drive the OpenAI-compatible handler, not Qwen, but the import is
# eager, so the dep must be present.
ENV PIP_DISABLE_PIP_VERSION_CHECK=1
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries=5 --timeout=120 \
           --index-url https://download.pytorch.org/whl/cpu torch \
      && pip install --no-cache-dir --retries=5 --timeout=120 \
           "bfcl_eval @ git+https://github.com/ShishirPatil/gorilla.git@${BENCHMARK_VERSION}#subdirectory=berkeley-function-call-leaderboard" \
           soundfile \
      ; then ok=1; break; fi; \
      echo "pip attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "pip dependency install failed after 5 tries" >&2; exit 1; }

# ── Build the stable index → (category, bfcl_id) map ──────────────────────
# The runner selects one task by its sequential integer EVAL_TASK_ID, but BFCL
# addresses tasks by its own string id (e.g. "simple_python_0") within a
# category. Assemble the six non-live AST categories in canonical order
# (category order × in-file order, no shuffle) and record one row per task as
# /tasks/all.jsonl: {id, category, bfcl_id}. Read straight from the installed
# package's bundled data so the map matches the harness that will grade it.
# Assert 1150 so a dataset change is caught at build. Root-only (§5): the agent
# must never read the task store.
RUN python3 <<'PYEOF'
import json, os
from importlib import resources

CATEGORIES = [
    "simple_python", "simple_java", "simple_javascript",
    "multiple", "parallel", "parallel_multiple",
]
EXPECTED = {
    "simple_python": 400, "simple_java": 100, "simple_javascript": 50,
    "multiple": 200, "parallel": 200, "parallel_multiple": 200,
}

data_dir = resources.files("bfcl_eval") / "data"
os.makedirs("/tasks", exist_ok=True)
rows = []
for category in CATEGORIES:
    path = data_dir / f"BFCL_v4_{category}.json"
    ids = []
    with path.open() as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            ids.append(json.loads(line)["id"])
    assert len(ids) == EXPECTED[category], \
        f"{category}: expected {EXPECTED[category]} entries, got {len(ids)}"
    for bfcl_id in ids:
        rows.append({"category": category, "bfcl_id": bfcl_id})

assert len(rows) == 1150, f"expected 1150 tasks, got {len(rows)}"
with open("/tasks/all.jsonl", "w") as out:
    for i, row in enumerate(rows):
        out.write(json.dumps({"id": i, **row}) + "\n")
print(f"Wrote {len(rows)} tasks to /tasks/all.jsonl")
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Shared task materializer (writes /tasks/$EVAL_TASK_ID/<field>.txt at runtime).
COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

WORKDIR /app
ENV EVAL_BENCHMARK=bfcl

# Single-task runner: resolve EVAL_TASK_ID → (category, bfcl_id), drive BFCL's
# generation + AST evaluation in-process against the gateway, and write the
# reward from the score file.
COPY run_bfcl.py /app/run_bfcl.py
RUN chmod 755 /app/run_bfcl.py

# The runner writes /logs/verifier/reward.txt itself; keep a fail-closed
# baseline grader so a run that never reaches the writer still scores 0.
RUN mkdir -p /logs/verifier && cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
