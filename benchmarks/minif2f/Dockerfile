# MiniF2F (formal math from olympiads and exams)
# 244 tasks (test split informal statements). Source: github yangky11/miniF2F
# Agent prints final answer to stdout. Test compares to expected answer.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="minif2f"
LABEL eval.benchmark.description="MiniF2F - formal-math competition problems (olympiad, AMC, AIME) with informal statements"
LABEL eval.benchmark.tasks="244"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c605019c4ebfda74d37bbb83e9c5774fc8b67c14"
LABEL eval.benchmark.url="https://github.com/yangky11/miniF2F"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2109.00110"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="c605019c4ebfda74d37bbb83e9c5774fc8b67c14"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
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

WORKDIR /app
ENV EVAL_BENCHMARK=minif2f

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competition math problem. Print only the final answer in its simplest form (a number or expression). Do not include any explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
