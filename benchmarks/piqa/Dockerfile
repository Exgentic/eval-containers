# PIQA (ybisk/piqa)
# 1838 tasks. Data: https://yonatanbisk.com/piqa (valid split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
#
# Note: the canonical HuggingFace ybisk/piqa dataset ships as a Python loader
# script, which can't be parquet-converted by HF. We fetch the original raw
# JSONL files hosted by the authors at yonatanbisk.com (same data).

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="piqa"
LABEL eval.benchmark.description="PIQA - physical commonsense reasoning (2-way multiple choice)"
LABEL eval.benchmark.tasks="1838"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="2e8ac2dffd59bac8c3c6714948f4c551a0848bb0"
LABEL eval.benchmark.url="https://yonatanbisk.com/piqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1911.11641"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="2e8ac2dffd59bac8c3c6714948f4c551a0848bb0"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, json, os
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid.jsonl', '/tmp/valid.jsonl')
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid-labels.lst', '/tmp/valid-labels.lst')
labels = [l.strip() for l in open('/tmp/valid-labels.lst') if l.strip()]
rows = [json.loads(l) for l in open('/tmp/valid.jsonl')]
assert len(rows) == len(labels), (len(rows), len(labels))
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i, (row, lab) in enumerate(zip(rows, labels)):
        problem = f"{row['goal']}\n\nA. {row['sol1']}\nB. {row['sol2']}"
        obj = {
            'id': str(i),
            'problem': problem,
            'answer': 'A' if lab == '0' else 'B',
        }
        out.write(json.dumps(obj) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN rm -f /tmp/valid.jsonl /tmp/valid-labels.lst
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=piqa

COPY --from=test-exact-match /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Pick the solution that best accomplishes the goal. Print only the letter of the correct answer (A or B), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
