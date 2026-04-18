# AssetOpsBench
# Industrial asset lifecycle management - 6-dimensional evaluation.
# Data: HuggingFace ibm-research/AssetOpsBench (scenario utterances).
# Evaluation requires the AssetOpsBench industrial agent environment
# (IoT, FMSR, TSFM, WorkOrder agents with domain-specific tools).
# Externally graded: test writes -1.

FROM quay.io/dock-eval/core/benchmark-base-external:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="assetopsbench"
LABEL dock.benchmark.description="AssetOpsBench - industrial asset operations 6-dimensional evaluation"
LABEL dock.benchmark.tasks="152"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c4a2ebd52436580723bf2c52b3aff87aa53bf999"
LABEL dock.benchmark.url="https://github.com/IBM/AssetOpsBench"
LABEL dock.benchmark.paper="https://huggingface.co/blog/ibm-research/assetopsbench-playground-on-hugging-face"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="c4a2ebd52436580723bf2c52b3aff87aa53bf999"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
# Uses the scenario utterances file which contains the full 152-task set.
RUN python3 <<'PYEOF'
import urllib.request, os, json
REV = "c4a2ebd52436580723bf2c52b3aff87aa53bf999"
url = f'https://huggingface.co/datasets/ibm-research/AssetOpsBench/resolve/{REV}/data/scenarios/all_utterance.jsonl'
urllib.request.urlretrieve(url, '/tmp/data.jsonl')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out, open('/tmp/data.jsonl') as fin:
    for line in fin:
        line = line.strip()
        if not line:
            continue
        src = json.loads(line)
        row = {
            'id': str(src.get('id')),
            'problem': str(src.get('text', '')),
            'category': str(src.get('category', '')),
            'entity': str(src.get('entity', '')),
            'type': str(src.get('type', '')),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=assetopsbench

# AssetOpsBench uses a 6-dimensional LLM-judge scoring across domain-specific
# agent trajectories (IoT, FMSR, TSFM, WorkOrder). The full evaluator requires
# the upstream AssetOpsBench runtime with live agent endpoints. Externally
# graded: always write -1 per compose/RULES.md rule 16.
# FUTURE: integrate the 6D scorer once the AssetOpsBench eval container is
# mirrored into the Dock registry.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
