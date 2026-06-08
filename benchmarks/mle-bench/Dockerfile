# syntax=docker/dockerfile:1
# MLE-bench (OpenAI)
# Kaggle-style ML engineering tasks. Upstream: github.com/openai/mle-bench.
#
# GENUINE BLOCKER: unlike the other SWE benchmarks, MLE-bench cannot be fully
# built from source alone. Each task's data is a Kaggle competition dataset
# that is hosted on kaggle.com and gated by:
#   1. A Kaggle API credential pair (KAGGLE_USERNAME + KAGGLE_KEY).
#   2. Explicit per-competition rules acceptance on kaggle.com (the user must
#      click through each competition's "I Understand and Accept" page).
# Upstream `mlebench prepare --competition-id <slug>` orchestrates the
# download+split into /home/data (read-only) and /private (answers), but it
# cannot bypass these requirements.
#
# This Dockerfile therefore has TWO builds:
#   (1) Shared base image "mlebench-env" (public, fully open-source). Pre-build
#       once with: scripts/build-mle-bench.sh
#   (2) Per-task image (this file) which uses mlebench to prepare a specific
#       competition. Callers MUST pass their Kaggle credentials as build
#       secrets (--secret id=kaggle,src=$HOME/.kaggle/kaggle.json) and MUST
#       have already accepted the competition rules on kaggle.com.
#
# For our representative task (spaceship-titanic) this is straightforward:
# Kaggle's spaceship-titanic is a free, public "Getting Started" competition.

ARG EVAL_TASK_ID
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM mlebench-env:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mle-bench"
LABEL eval.benchmark.upstream_base="mlebench-env:latest (locally built from openai/mle-bench)"
LABEL eval.benchmark.description="MLE-bench - Kaggle-style ML engineering tasks (OpenAI)"
LABEL eval.benchmark.tasks="75"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.url="https://github.com/openai/mle-bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2410.07095"
LABEL eval.benchmark.requires="Kaggle API credentials + per-competition rules acceptance"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="0.1.0"
ENV EVAL_BENCHMARK=mle-bench

RUN apt-get update && apt-get install -y --no-install-recommends curl jq && rm -rf /var/lib/apt/lists/*

ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

# Install mlebench into a dedicated root-only location so the grader is
# tamper-resistant. (The base image already has heavy ML deps.)
RUN pip install --no-cache-dir --target /tests/deps mlebench==1.4.0

# Prepare the competition's data + answers. Requires Kaggle creds mounted as
# a build secret; the user must have accepted the competition's rules.
# Without creds, we fall back to a placeholder so the image still builds and
# the missing-data error is reported at run time.
RUN --mount=type=secret,id=kaggle,target=/root/.kaggle/kaggle.json,required=false \
    mkdir -p /home/data /private /tasks && PYTHONPATH=/tests/deps python3 - <<'PY'
import os, sys, shutil, pathlib, json
sys.path.insert(0, "/tests/deps")
tid = os.environ["EVAL_TASK_ID"]
try:
    from mlebench.registry import registry
    from mlebench.data import download_and_prepare_dataset
    comp = registry.get_competition(tid)
    download_and_prepare_dataset(comp)
    shutil.copytree(str(comp.public_dir), "/home/data", dirs_exist_ok=True)
    shutil.copytree(str(comp.private_dir), "/private", dirs_exist_ok=True)
    print("prepared", tid)
except Exception as e:
    print("WARN: data prep failed:", e)
    pathlib.Path("/home/data/MISSING.txt").write_text(str(e))

desc = ""
for p in ("/private/description.md", "/home/data/description.md"):
    pp = pathlib.Path(p)
    if pp.exists():
        desc = pp.read_text()
        break
row = {"id": tid, "problem": desc or f"MLE-bench task {tid}"}
pathlib.Path("/tasks/all.jsonl").write_text(json.dumps(row) + "\n")
PY

RUN chmod 600 /tasks/all.jsonl

# Grader: mlebench.grade scores submission.csv against held-out answers.
RUN cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
PYTHONPATH=/tests/deps python3 /tests/grade.py > /logs/verifier/reward.txt 2>/dev/null || echo 0 > /logs/verifier/reward.txt
TEST
RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'GRADE'
import sys, os, json
sys.path.insert(0, '/tests/deps')
os.makedirs('/output/agent', exist_ok=True)
submission = '/home/submission/submission.csv'
if not os.path.exists(submission):
    for candidate in ('/home/submission.csv', '/app/submission.csv', '/workspace/submission.csv'):
        if os.path.exists(candidate):
            submission = candidate; break
if not os.path.exists(submission):
    print(0); sys.exit(0)
try:
    from mlebench.grade import grade_csv
    from mlebench.registry import registry
    competition = registry.get_competition(os.environ.get('EVAL_TASK_ID'))
    report = grade_csv(submission, competition)
    print(float(bool(getattr(report, 'any_medal', False) or getattr(report, 'gold_medal', False))))
except Exception as e:
    print(0, file=sys.stderr)
    print(0)
GRADE

RUN chmod -R 700 /tests && chown -R root:root /tests

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
mkdir -p /home/submission
chown -R agent:agent /home/submission /home/data 2>/dev/null || true

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are an ML engineer. The competition data is mounted at /home/data (read-only). Train a model and write your predictions to /home/submission/submission.csv. You may install packages and use the network.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
