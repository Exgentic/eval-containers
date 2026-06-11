# MMLU-Pro (TIGER-Lab/MMLU-Pro)
# 12032 tasks. Data: HuggingFace TIGER-Lab/MMLU-Pro
# Agent prints answer to stdout. Test compares stdout to expected answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmlu-pro"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MMLU-Pro - graduate-level knowledge, multiple choice"
LABEL eval.benchmark.tasks="12032"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="54611cde22c74cca43dd78732198de6abe971398"
LABEL eval.benchmark.url="https://github.com/TIGER-AI-Lab/MMLU-Pro"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mmlu_pro"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.01574"

# Single-JSONL task layout. Root-only; runtime entrypoint materializes the
# requested task's fields into /tasks/$EVAL_TASK_ID/ on demand.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, string, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/TIGER-Lab/MMLU-Pro/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = list(string.ascii_uppercase)
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        opts = t['options'][i].as_py()
        choices = '\n'.join(f'{letters[j]}. {opt}' for j, opt in enumerate(opts))
        problem = f"{t['question'][i]}\n\n{choices}"
        row = {
            'id': str(t['question_id'][i]),
            'problem': problem,
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mmlu-pro

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, D, E, F, G, H, I, or J), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
