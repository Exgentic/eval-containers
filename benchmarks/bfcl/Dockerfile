# syntax=docker/dockerfile:1
# BFCL (Berkeley Function Calling Leaderboard)
# ~2000 tasks (simple/multiple/parallel categories). Data: HuggingFace gorilla-llm
# Agent outputs function call(s). Test compares to ground truth.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="bfcl"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Berkeley Function Calling Leaderboard"
LABEL eval.benchmark.tasks="2000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/ShishirPatil/gorilla"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/bfcl"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bfcl"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.06840"

# Fetch task and answer files from HuggingFace
RUN mkdir -p /tmp/bfcl/tasks /tmp/bfcl/answers
RUN for cat in simple multiple parallel; do \
      curl -sL "https://huggingface.co/datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard/resolve/main/BFCL_v3_${cat}.json" \
        -o /tmp/bfcl/tasks/${cat}.json && \
      curl -sL "https://huggingface.co/datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard/resolve/main/possible_answer/BFCL_v3_${cat}.json" \
        -o /tmp/bfcl/answers/${cat}.json; \
    done

# Single-JSONL task layout. Agent never sees /tasks/all.jsonl (root-only).
RUN python3 <<'PYEOF'
import json, os
os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for cat in ['simple', 'multiple', 'parallel']:
        tasks = [json.loads(l) for l in open(f'/tmp/bfcl/tasks/{cat}.json')]
        answers = [json.loads(l) for l in open(f'/tmp/bfcl/answers/{cat}.json')]
        answer_map = {a['id']: a for a in answers}
        for t in tasks:
            funcs = json.dumps(t.get('function', []), indent=2)
            q = t.get('question', [[]])
            msgs_list = q[0] if q and isinstance(q[0], list) else q
            msgs = '\n'.join(f"{m['role']}: {m['content']}" for m in msgs_list if isinstance(m, dict))
            ans = answer_map.get(t['id'], {})
            row = {
                'id': t['id'],
                'problem': f"Given these function definitions:\n{funcs}\n\n{msgs}",
                'answer': json.dumps(ans.get('ground_truth', [])),
            }
            out.write(json.dumps(row) + '\n')
            idx += 1
print(f'Extracted {idx} tasks')
PYEOF

RUN rm -rf /tmp/bfcl
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=bfcl

# Test: exact match on agent stdout (simplified — full BFCL uses AST comparison)
COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Call the correct function(s) based on the conversation. Print ONLY the function call(s) as JSON, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
