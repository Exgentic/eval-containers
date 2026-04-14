# Mind2Web (OSU NLP)
# Generalist web agent tasks across 137 websites. Data: HuggingFace osunlp/Mind2Web
# Each task is a natural language instruction to perform on a real web page.
# Upstream dataset contains pre-recorded HTML snapshots + gold action traces; no live
# site is required. Grading is externally handled (element/action match against gold
# trace is tightly coupled to upstream tooling). This benchmark captures the agent's
# output trace and writes -1 per RULES.md section Scoring.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mind2web"
LABEL dock.benchmark.description="Mind2Web - generalist web agent instruction following"
LABEL dock.benchmark.tasks="1009"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="17ece8eb89862368edc0cc806acee6fca5163474"
LABEL dock.benchmark.url="https://github.com/OSU-NLP-Group/Mind2Web"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2306.06070"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch the first parquet shard (task metadata; full HTML traces live in other shards).
# The instruction text ("confirmed_task") + action_reprs gold trace is enough to grade
# externally and to expose the task to the agent.
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
BASE = 'https://huggingface.co/api/datasets/osunlp/Mind2Web/parquet/default/train'
idx = 0
for shard in range(8):
    url = f'{BASE}/{shard}.parquet'
    local = f'/tmp/s{shard}.parquet'
    urllib.request.urlretrieve(url, local)
    t = pq.read_table(local, columns=['annotation_id','website','domain','subdomain','confirmed_task','action_reprs'])
    for i in range(len(t)):
        os.makedirs(f'/tasks/{idx}', exist_ok=True)
        with open(f'/tasks/{idx}/id.txt','w') as f: f.write(str(t['annotation_id'][i]))
        with open(f'/tasks/{idx}/problem.txt','w') as f: f.write(str(t['confirmed_task'][i]))
        meta = {
            'website':   str(t['website'][i]),
            'domain':    str(t['domain'][i]),
            'subdomain': str(t['subdomain'][i]),
        }
        json.dump(meta, open(f'/tasks/{idx}/meta.json','w'))
        actions = [str(a) for a in t['action_reprs'][i].as_py()]
        json.dump(actions, open(f'/tasks/{idx}/gold_trace.json','w'))
        idx += 1
    os.remove(local)
print(f'Extracted {idx} tasks')
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=mind2web

# Test: externally graded. Preserve agent trace, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
# Agent stdout is the proposed action trace; keep it for offline grading.
cp /output/agent/stdout.log /output/task/agent_trace.log 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  META=$(cat /tasks/$TASK_ID/meta.json)
  export TASK="You are a generalist web agent. Produce a step-by-step action trace that
would complete the following task on the target website. Each line of your output
MUST describe one UI action in the form: [element_type] element_label -> ACTION[: value]

Target site metadata: $META

Task: $(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
