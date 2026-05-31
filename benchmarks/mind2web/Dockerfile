# Mind2Web (OSU NLP)
# Generalist web agent tasks across 137 websites. Data: HuggingFace osunlp/Mind2Web
# Each task is a natural language instruction to perform on a real web page.
# Upstream dataset contains pre-recorded HTML snapshots + gold action traces; no live
# site is required. Grading is externally handled (element/action match against gold
# trace is tightly coupled to upstream tooling). This benchmark captures the agent's
# output trace and writes -1 per RULES.md section Scoring.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mind2web"
LABEL eval.benchmark.description="Mind2Web - generalist web agent instruction following"
LABEL eval.benchmark.tasks="1009"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="17ece8eb89862368edc0cc806acee6fca5163474"
LABEL eval.benchmark.url="https://github.com/OSU-NLP-Group/Mind2Web"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2306.06070"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="17ece8eb89862368edc0cc806acee6fca5163474"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/ and renames meta/gold_trace to .json.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
BASE = 'https://huggingface.co/api/datasets/osunlp/Mind2Web/parquet/default/train'
os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for shard in range(8):
        url = f'{BASE}/{shard}.parquet'
        local = f'/tmp/s{shard}.parquet'
        urllib.request.urlretrieve(url, local)
        t = pq.read_table(local, columns=['annotation_id','website','domain','subdomain','confirmed_task','action_reprs'])
        for i in range(len(t)):
            meta = {
                'website':   str(t['website'][i]),
                'domain':    str(t['domain'][i]),
                'subdomain': str(t['subdomain'][i]),
            }
            actions = [str(a) for a in t['action_reprs'][i].as_py()]
            row = {
                'id': str(t['annotation_id'][i]),
                'problem': str(t['confirmed_task'][i]),
                'meta': json.dumps(meta),
                'gold_trace': json.dumps(actions),
            }
            out.write(json.dumps(row) + '\n')
            idx += 1
        os.remove(local)
print(f'Extracted {idx} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mind2web

# Test: externally graded. Preserve agent trace, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
# Agent stdout is the proposed action trace; keep it for offline grading.
cp /output/agent/stdout.log /output/task/agent_trace.log 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ]; then
  TDIR="/tasks/$EVAL_TASK_ID"
  [ -f "$TDIR/meta.txt" ] && mv "$TDIR/meta.txt" "$TDIR/meta.json" && chmod 600 "$TDIR/meta.json"
  [ -f "$TDIR/gold_trace.txt" ] && mv "$TDIR/gold_trace.txt" "$TDIR/gold_trace.json" && chmod 600 "$TDIR/gold_trace.json"
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  META=$(cat /tasks/$EVAL_TASK_ID/meta.json)
  export TASK="You are a generalist web agent. Produce a step-by-step action trace that
would complete the following task on the target website. Each line of your output
MUST describe one UI action in the form: [element_type] element_label -> ACTION[: value]

Target site metadata: $META

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
