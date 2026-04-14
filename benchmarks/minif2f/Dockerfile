# MiniF2F (formal math from olympiads and exams)
# 244 tasks (test split informal statements). Source: github yangky11/miniF2F
# Agent prints final answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="minif2f"
LABEL dock.benchmark.description="MiniF2F - formal-math competition problems (olympiad, AMC, AIME) with informal statements"
LABEL dock.benchmark.tasks="244"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c605019c4ebfda74d37bbb83e9c5774fc8b67c14"
LABEL dock.benchmark.url="https://github.com/yangky11/miniF2F"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2109.00110"

RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*
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

task_idx = 0
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
    os.makedirs(f'/tasks/{task_idx}', exist_ok=True)
    open(f'/tasks/{task_idx}/id.txt', 'w').write(pname)
    open(f'/tasks/{task_idx}/problem.txt', 'w').write(problem)
    open(f'/tasks/{task_idx}/answer.txt', 'w').write(answer)
    open(f'/tasks/{task_idx}/informal_proof.txt', 'w').write(proof)
    task_idx += 1
print('total tasks:', task_idx)
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=minif2f

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competition math problem. Print only the final answer in its simplest form (a number or expression). Do not include any explanation.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
