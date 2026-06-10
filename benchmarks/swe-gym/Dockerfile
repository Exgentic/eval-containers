# SWE-Gym (Columbia + CMU)
# 2438 Python SWE tasks sourced from 11 repos following the SWE-bench data-collection
# procedure. Data: HuggingFace SWE-Gym/SWE-Gym
#
# Each task has (repo, base_commit, problem_statement, patch, test_patch,
# FAIL_TO_PASS, PASS_TO_PASS). Full grading requires checking out the repo at
# base_commit, applying the test_patch, running the specified tests and parsing the
# results - which is what the upstream SWE-Gym / SWE-bench harness does. This image
# exposes the problem statement to the agent, captures the produced patch, and writes
# -1 (externally graded).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-gym"
LABEL eval.benchmark.description="SWE-Gym - 2438 Python SWE tasks for agent training / eval"
LABEL eval.benchmark.tasks="2438"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="bb94ed9e39bbeb96a7fcbfb533b80f25a7fd59cb"
LABEL eval.benchmark.url="https://github.com/SWE-Gym/SWE-Gym"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2412.21139"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/SWE-Gym/SWE-Gym/parquet/default/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['instance_id'][i]),
            'problem': str(t['problem_statement'][i]),
            'repo': str(t['repo'][i]),
            'base_commit': str(t['base_commit'][i]),
            'version': str(t['version'][i]),
            'fail_to_pass': list(t['FAIL_TO_PASS'][i].as_py() or []),
            'pass_to_pass': list(t['PASS_TO_PASS'][i].as_py() or []),
            'test_patch': str(t['test_patch'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=swe-gym

# Test: externally graded. Preserve agent patch/transcript for offline SWE-bench grading.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_transcript.log 2>/dev/null || true
if [ -f /output/agent/patch.diff ]; then
  cp /output/agent/patch.diff /output/task/patch.diff
fi
echo -1 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  REPO=$(cat /tasks/$EVAL_TASK_ID/repo.txt)
  COMMIT=$(cat /tasks/$EVAL_TASK_ID/base_commit.txt)
  export TASK="Fix this GitHub issue in the Python repository $REPO at commit $COMMIT.
Produce a unified diff patch that resolves the bug. Print the patch to stdout.
Do NOT modify any test files.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
