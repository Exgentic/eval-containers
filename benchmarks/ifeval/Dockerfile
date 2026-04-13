# IFEval (google/IFEval)
# 541 tasks. Data: HuggingFace google/IFEval
# Agent follows a prompt with verifiable constraints. Test checks instruction compliance.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="ifeval"
LABEL dock.benchmark.description="IFEval - instruction following with verifiable constraints"
LABEL dock.benchmark.tasks="541"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="966cd89545d6b6acfd7638bc708b98261ca58e84"
LABEL dock.benchmark.url="https://github.com/google-research/google-research/tree/master/instruction_following_eval"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2311.07911"

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks (prompt, instruction_id_list, kwargs per task)
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/google/IFEval/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['key'][i]))
    open(f'/tasks/{i}/prompt.txt', 'w').write(str(t['prompt'][i]))
    # Store instruction IDs and kwargs as JSON for the verifier
    instruction_ids = [str(x) for x in t['instruction_id_list'][i].as_py()]
    open(f'/tasks/{i}/instruction_ids.json', 'w').write(json.dumps(instruction_ids))
    kwargs_list = t['kwargs'][i].as_py()
    # Each element is a dict serialized as JSON string; parse then re-dump
    parsed_kwargs = []
    for kw in kwargs_list:
        if isinstance(kw, str):
            parsed_kwargs.append(json.loads(kw))
        elif isinstance(kw, dict):
            parsed_kwargs.append(kw)
        else:
            parsed_kwargs.append({})
    open(f'/tasks/{i}/kwargs.json', 'w').write(json.dumps(parsed_kwargs))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow

# Install official IFEval verification code from google-research
RUN pip install --no-cache-dir absl-py langdetect nltk immutabledict
RUN python3 -c "import nltk; nltk.download('punkt_tab', quiet=True)"
RUN git clone --depth 1 https://github.com/google-research/google-research.git /tmp/google-research \
    && cp -r /tmp/google-research/instruction_following_eval /opt/instruction_following_eval \
    && rm -rf /tmp/google-research \
    && touch /opt/instruction_following_eval/__init__.py

RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=ifeval

# Custom test: verify instruction following using official IFEval logic
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
RESPONSE=$(cat /output/agent/stdout.log 2>/dev/null)

if [ -z "$RESPONSE" ]; then
  echo 0.0 > /logs/verifier/reward.txt
  exit 0
fi

# Run the verifier
python3 /tests/verify.py "$TASK_DIR" "$RESPONSE" > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST
RUN chmod +x /tests/test.sh

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

def test_instruction_following_loose(instruction_ids, kwargs_list, response):
    """Loose check: try response variants (strip markdown, first/last line removal)."""
    total = len(instruction_ids)
    if total == 0:
        return 1.0

    # Generate response variants for loose evaluation
    response_stripped = response.strip()
    lines = response_stripped.split('\n')
    variants = [response_stripped]
    # Remove asterisks (markdown bold/italic)
    no_asterisks = response_stripped.replace('*', '')
    variants.append(no_asterisks)
    # Remove first line
    if len(lines) > 1:
        variants.append('\n'.join(lines[1:]))
        variants.append('\n'.join(lines[1:]).replace('*', ''))
    # Remove last line
    if len(lines) > 1:
        variants.append('\n'.join(lines[:-1]))
        variants.append('\n'.join(lines[:-1]).replace('*', ''))
    # Remove first and last
    if len(lines) > 2:
        variants.append('\n'.join(lines[1:-1]))
        variants.append('\n'.join(lines[1:-1]).replace('*', ''))

    followed = 0
    for inst_id, kwargs in zip(instruction_ids, kwargs_list):
        inst_cls = instructions_registry.INSTRUCTION_DICT.get(inst_id)
        if inst_cls is None:
            followed += 1
            continue
        clean_kwargs = {k: v for k, v in kwargs.items() if v is not None}

        inst_followed = False
        for variant in variants:
            if not variant.strip():
                continue
            instruction = inst_cls(inst_id)
            instruction.build_description(**clean_kwargs)
            if instruction.check_following(variant):
                inst_followed = True
                break
        if inst_followed:
            followed += 1

    return followed / total

task_dir = sys.argv[1]
response = sys.argv[2]

instruction_ids = json.loads(open(os.path.join(task_dir, 'instruction_ids.json')).read())
kwargs_list = json.loads(open(os.path.join(task_dir, 'kwargs.json')).read())

# Use strict scoring as the primary reward
score = test_instruction_following(instruction_ids, kwargs_list, response)
print(f"{score:.4f}")
PYEOF

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$TASK_ID/prompt.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
