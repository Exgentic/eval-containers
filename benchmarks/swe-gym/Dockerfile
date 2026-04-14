# SWE-Gym (Columbia + CMU)
# 2438 Python SWE tasks sourced from 11 repos following the SWE-bench data-collection
# procedure. Data: HuggingFace SWE-Gym/SWE-Gym
#
# Each task has (repo, base_commit, problem_statement, patch, test_patch,
# FAIL_TO_PASS, PASS_TO_PASS). Full grading requires checking out the repo at
# base_commit, applying the test_patch, running the specified tests and parsing the
# results — which is what the upstream SWE-Gym / SWE-bench harness does. This image
# exposes the problem statement to the agent, captures the produced patch, and writes
# -1 (externally graded).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="swe-gym"
LABEL dock.benchmark.description="SWE-Gym - 2438 Python SWE tasks for agent training / eval"
LABEL dock.benchmark.tasks="2438"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="bb94ed9e39bbeb96a7fcbfb533b80f25a7fd59cb"
LABEL dock.benchmark.url="https://github.com/SWE-Gym/SWE-Gym"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2412.21139"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/SWE-Gym/SWE-Gym/parquet/default/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt','w').write(str(t['instance_id'][i]))
    open(f'/tasks/{i}/problem.txt','w').write(str(t['problem_statement'][i]))
    config = {
        'instance_id':       str(t['instance_id'][i]),
        'repo':              str(t['repo'][i]),
        'base_commit':       str(t['base_commit'][i]),
        'version':           str(t['version'][i]),
        'FAIL_TO_PASS':      list(t['FAIL_TO_PASS'][i].as_py() or []),
        'PASS_TO_PASS':      list(t['PASS_TO_PASS'][i].as_py() or []),
        'test_patch':        str(t['test_patch'][i]),
    }
    json.dump(config, open(f'/tasks/{i}/config.json','w'))
print(f'Extracted {len(t)} tasks')
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=swe-gym

# Test: externally graded. Preserve agent patch/transcript for offline SWE-bench grading.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_transcript.log 2>/dev/null || true
if [ -f /output/agent/patch.diff ]; then
  cp /output/agent/patch.diff /output/task/patch.diff
fi
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  CONFIG=$(cat /tasks/$TASK_ID/config.json)
  REPO=$(python3 -c "import json; print(json.loads('''$CONFIG''')['repo'])")
  COMMIT=$(python3 -c "import json; print(json.loads('''$CONFIG''')['base_commit'])")
  export TASK="Fix this GitHub issue in the Python repository $REPO at commit $COMMIT.
Produce a unified diff patch that resolves the bug. Print the patch to stdout.
Do NOT modify any test files.

$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
