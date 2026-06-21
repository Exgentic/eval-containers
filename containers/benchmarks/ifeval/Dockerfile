# IFEval (google/IFEval)
# 541 tasks. Data: HuggingFace google/IFEval
# Agent follows a prompt with verifiable constraints. Test checks instruction compliance.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data

# Loader runs pyarrow, so it stays in the discarded hf builder (size win; QEMU-build
# still needs duckdb, a later step). The official IFEval verifier is cloned here
# (where git is available); nltk/absl + the verifier run on python-slim at runtime.

# Single-JSONL task layout (see benchmarks/RULES.md #22).
# instruction_ids and kwargs are stored as JSON strings (field names
# instruction_ids_json, kwargs_json); the verifier parses them.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/google/IFEval/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        instruction_ids = [str(x) for x in t['instruction_id_list'][i].as_py()]
        kwargs_list = t['kwargs'][i].as_py()
        parsed_kwargs = []
        for kw in kwargs_list:
            if isinstance(kw, str):
                parsed_kwargs.append(json.loads(kw))
            elif isinstance(kw, dict):
                parsed_kwargs.append(kw)
            else:
                parsed_kwargs.append({})
        row = {
            'id': str(t['key'][i]),
            'problem': str(t['prompt'][i]),
            'prompt': str(t['prompt'][i]),
            'instruction_ids_json': json.dumps(instruction_ids),
            'kwargs_json': json.dumps(parsed_kwargs),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Clone the official IFEval verifier in the builder (needs git).
RUN git clone --depth 1 https://github.com/google-research/google-research.git /tmp/google-research \
    && cp -r /tmp/google-research/instruction_following_eval /opt/instruction_following_eval \
    && rm -rf /tmp/google-research \
    && touch /opt/instruction_following_eval/__init__.py

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="ifeval"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="IFEval - instruction following with verifiable constraints"
LABEL eval.benchmark.tasks="541"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="966cd89545d6b6acfd7638bc708b98261ca58e84"
LABEL eval.benchmark.url="https://github.com/google-research/google-research/tree/master/instruction_following_eval"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/ifeval"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2311.07911"

# Runtime deps for the official IFEval verifier.
RUN pip install --no-cache-dir absl-py==2.1.0 langdetect==1.0.9 nltk==3.9.1 immutabledict==4.2.0
RUN python3 -c "import nltk; nltk.download('punkt_tab', quiet=True)"
COPY --from=data /opt/instruction_following_eval /opt/instruction_following_eval
COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=ifeval

# Custom test: verify instruction following using official IFEval logic
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
RESPONSE=$(cat /output/agent/stdout.log 2>/dev/null)

if [ -z "$RESPONSE" ]; then
  echo 0.0 > /logs/verifier/reward.txt
  exit 0
fi

# Run the verifier
python3 /tests/verify.py "$TASK_DIR" "$RESPONSE" > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST

RUN chmod +x /grade.sh

# Python verifier that uses the official IFEval instruction checks
RUN cat > /tests/verify.py <<'PYEOF'
import sys, json, os

sys.path.insert(0, '/opt')
from instruction_following_eval import instructions_registry

def test_instruction_following(instruction_ids, kwargs_list, response):
    """Check each instruction against the response. Returns fraction followed."""
    total = len(instruction_ids)
    if total == 0:
        return 1.0

    followed = 0
    for inst_id, kwargs in zip(instruction_ids, kwargs_list):
        inst_cls = instructions_registry.INSTRUCTION_DICT.get(inst_id)
        if inst_cls is None:
            # Unknown instruction type, skip (count as followed)
            followed += 1
            continue
        # Filter out None values from kwargs
        clean_kwargs = {k: v for k, v in kwargs.items() if v is not None}
        instruction = inst_cls(inst_id)
        instruction.build_description(**clean_kwargs)

        # Strict check
        if response.strip() and instruction.check_following(response):
            followed += 1

    return followed / total

task_dir = sys.argv[1]
response = sys.argv[2]

instruction_ids = json.loads(open(os.path.join(task_dir, 'instruction_ids_json.txt')).read())
kwargs_list = json.loads(open(os.path.join(task_dir, 'kwargs_json.txt')).read())

# Use strict scoring as the primary reward
score = test_instruction_following(instruction_ids, kwargs_list, response)
print(f"{score:.4f}")
PYEOF

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/prompt.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
