# VisualWebArena (CMU + others)
# 910 tasks across classifieds (234), reddit (210), shopping (466).
# Multimodal web agent benchmark with real website sidecars + mitmproxy HAR capture.
# Evaluation is externally handled by the upstream visualwebarena harness; this image
# captures the agent's final answer + network HAR and writes -1.

FROM quay.io/eval-containers/core/benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="visualwebarena"
LABEL eval.benchmark.description="VisualWebArena - multimodal web agent tasks with sidecars"
LABEL eval.benchmark.tasks="910"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/web-arena-x/visualwebarena"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2401.13649"
ENV EVAL_BENCHMARK_VERSION_DEFAULT=""

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import json, os, urllib.request
BASE = 'https://raw.githubusercontent.com/web-arena-x/visualwebarena/main/config_files/vwa'
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for site in ('classifieds', 'reddit', 'shopping'):
        url = f'{BASE}/test_{site}.raw.json'
        local = f'/tmp/{site}.json'
        urllib.request.urlretrieve(url, local)
        tasks = json.load(open(local))
        for t in tasks:
            row = {
                'id': f"{site}/{t['task_id']}",
                'problem': t['intent'],
                'task': t,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove(local)
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=visualwebarena

# Test: externally graded. Capture agent output + HAR for offline scoring.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_response.txt 2>/dev/null || true
cp /output/agent/network.har /output/task/network.har 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  INTENT=$(cat /tasks/$EVAL_TASK_ID/problem.txt)
  export TASK="You are a multimodal web browsing agent. Complete this task by interacting
with the provided sidecar websites. Use HTTP_PROXY=http://proxy:8080 for all requests.

Task: $INTENT

Available sites:
- Classifieds:      http://classifieds:80
- Reddit:           http://reddit:80
- Shopping (OneStop): http://shopping:80

Credentials:
- Classifieds: blake.sullivan@gmail.com / Password.123
- Reddit:      MarvelsGrantMan136 / test1234
- Shopping:    emma.lopez@gmail.com / Password.123

When done, print your final answer (or the resulting URL for navigation tasks)."
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
