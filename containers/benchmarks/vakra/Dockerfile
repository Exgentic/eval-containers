# VAKRA
# Multi-hop, multi-source enterprise agent benchmark.
# Data: HuggingFace ibm-research/VAKRA.
# Full evaluation requires the VAKRA executable environment (8,000+ locally
# hosted APIs, SQLite databases, document collections). Externally graded.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="vakra"
LABEL eval.benchmark.description="VAKRA - multi-hop multi-source tool-calling benchmark"
LABEL eval.benchmark.tasks="28"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="1511b3a6ce0bb8df8aca2ae1b578510e150b6b7e"
LABEL eval.benchmark.url="https://github.com/IBM/vakra"
LABEL eval.benchmark.paper="https://huggingface.co/datasets/ibm-research/VAKRA"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
# We ingest the capability_3 multi-hop reasoning split (single-turn queries,
# 1-3 API chain). The per-domain JSON files each contain one dialogue; we
# flatten them into one row per domain. The heavier capability_4 (multi-turn,
# multi-source with policies) is deferred - see FUTURE below.
RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, os, json
REV = "1511b3a6ce0bb8df8aca2ae1b578510e150b6b7e"
API = f'https://huggingface.co/api/datasets/ibm-research/VAKRA/tree/{REV}/train/capability_3_multihop_reasoning/input'
with urllib.request.urlopen(API) as r:
    entries = json.load(r)
files = [e['path'] for e in entries if e['path'].endswith('.json')]
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for p in sorted(files):
        url = f'https://huggingface.co/datasets/ibm-research/VAKRA/resolve/{REV}/{p}'
        with urllib.request.urlopen(url) as r:
            data = json.load(r)
        if isinstance(data, list):
            items = data
        else:
            items = [data]
        for item in items:
            uuid = item.get('uuid') or os.path.basename(p).replace('.json', '')
            domain = item.get('domain', os.path.basename(p).replace('.json', ''))
            turns = item.get('dialogue', {}).get('turns', [])
            query = turns[0].get('query', '') if turns else ''
            row = {
                'id': uuid,
                'problem': query,
                'domain': domain,
                'num_turns': item.get('num_turns', 1),
            }
            out.write(json.dumps(row) + '\n')
            count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=vakra

# VAKRA scoring is trace-level - it replays the agent's API call sequence
# against the locally hosted LiveAPIBench runtime and BIRD-SQL databases.
# This requires the upstream VAKRA environment (8k+ APIs + SQLite DBs),
# which is not yet mirrored into Eval Containers. Externally graded per
# compose/RULES.md rule 16.
# FUTURE: ship the VAKRA API runtime as a sidecar; then swap in the
# trace-verification scorer and the capability_4 multi-turn split.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
