# MATH (Hendrycks competition mathematics)
# 5000 tasks (7 subjects x test split). Data: HuggingFace EleutherAI/hendrycks_math
# Agent prints answer to stdout. Test compares to boxed expected answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="math"
LABEL eval.benchmark.description="MATH - competition mathematics problems across 7 subjects (Hendrycks et al.)"
LABEL eval.benchmark.tasks="5000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="21a5633873b6a120296cce3e2df9d5550074f4a3"
LABEL eval.benchmark.url="https://huggingface.co/datasets/EleutherAI/hendrycks_math"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2103.03874"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="21a5633873b6a120296cce3e2df9d5550074f4a3"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json

SUBJECTS = ['algebra', 'counting_and_probability', 'geometry',
            'intermediate_algebra', 'number_theory', 'prealgebra', 'precalculus']

def extract_boxed(s):
    idx = s.rfind('\\boxed{')
    if idx < 0:
        return s.strip()
    i = idx + len('\\boxed{')
    depth = 1
    out = []
    while i < len(s) and depth > 0:
        c = s[i]
        if c == '{':
            depth += 1
            out.append(c)
        elif c == '}':
            depth -= 1
            if depth == 0:
                break
            out.append(c)
        else:
            out.append(c)
        i += 1
    return ''.join(out).strip()

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for subject in SUBJECTS:
        url = f'https://huggingface.co/api/datasets/EleutherAI/hendrycks_math/parquet/{subject}/test/0.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            row = {
                'id': f'{subject}_{i}',
                'problem': str(t['problem'][i]),
                'answer': extract_boxed(str(t['solution'][i])),
                'subject': subject,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print('total tasks:', count)
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=math

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this math problem. Print only the final answer in its simplest form (a number, fraction, or LaTeX expression as it would appear inside \\boxed{}). Do not include any explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
