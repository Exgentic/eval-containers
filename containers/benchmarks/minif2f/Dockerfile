# MiniF2F (formal math from olympiads and exams)
# 244 tasks (test split informal statements). Source: github yangky11/miniF2F
# Agent prints final answer to stdout. Test compares to expected answer.
#
# Source is a github tarball parsed with regex — not duckdb-loadable. The stdlib
# loader (urllib + tarfile + re, no pyarrow → QEMU-safe) runs in a discarded
# benchmark-base-hf builder; the runtime is slim (no python, no shipped pyarrow).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, tarfile, io, os, json, re

SHA = 'c605019c4ebfda74d37bbb83e9c5774fc8b67c14'
url = f'https://codeload.github.com/yangky11/miniF2F/tar.gz/{SHA}'
with urllib.request.urlopen(url) as r:
    data = r.read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode='r:gz')

prefix = f'miniF2F-{SHA}/informal/test/'
members = sorted(m for m in tf.getnames() if m.startswith(prefix) and m.endswith('.json'))

show_re = re.compile(r'[Ss]how that it is\s+([^.]+?)\.?\s*$')

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for name in members:
        f = tf.extractfile(name)
        if f is None:
            continue
        obj = json.loads(f.read().decode('utf-8'))
        stmt = obj.get('informal_statement', '').strip()
        proof = obj.get('informal_proof', '')
        pname = obj.get('problem_name', os.path.basename(name).removesuffix('.json'))
        m = show_re.search(stmt)
        if m:
            answer = m.group(1).strip()
            problem = show_re.sub('', stmt).strip()
        else:
            answer = ''
            problem = stmt
        row = {
            'id': pname,
            'problem': problem,
            'answer': answer,
            'informal_proof': proof,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print('total tasks:', count)
PYEOF

RUN chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="minif2f"
LABEL eval.benchmark.description="MiniF2F - formal-math competition problems (olympiad, AMC, AIME) with informal statements"
LABEL eval.benchmark.tasks="244"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c605019c4ebfda74d37bbb83e9c5774fc8b67c14"
LABEL eval.benchmark.url="https://github.com/yangky11/miniF2F"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2109.00110"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=minif2f
ENV EVAL_TASK_PROMPT="Solve this competition math problem. Print only the final answer in its simplest form (a number or expression). Do not include any explanation."
