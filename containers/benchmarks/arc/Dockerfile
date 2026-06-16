# ARC (AI2 Reasoning Challenge - Challenge set)
# 1172 tasks. Data: HuggingFace allenai/ai2_arc (ARC-Challenge/test)
# Agent prints answer letter to stdout. Test compares to expected letter.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="arc"
LABEL eval.benchmark.description="ARC Challenge - grade-school science multiple choice questions requiring reasoning"
LABEL eval.benchmark.tasks="1172"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="210d026faf9955653af8916fad021475a3f00453"
LABEL eval.benchmark.url="https://huggingface.co/datasets/allenai/ai2_arc"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1803.05457"

RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/ai2_arc/resolve/210d026faf9955653af8916fad021475a3f00453/ARC-Challenge/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        q = t['question'][i].as_py()
        choices = t['choices'][i].as_py()
        lines = [q, ""]
        for label, text in zip(choices['label'], choices['text']):
            lines.append(f"{label}. {text}")
        row = {
            'id': str(t['id'][i].as_py()),
            'problem': "\n".join(lines),
            'answer': str(t['answerKey'][i].as_py()),
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=arc

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this science multiple choice question. Print only the label of the correct answer (e.g., A), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
