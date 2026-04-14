# syntax=docker/dockerfile:1
# BFCL (Berkeley Function Calling Leaderboard)
# ~2000 tasks (simple/multiple/parallel categories). Data: HuggingFace gorilla-llm
# Agent outputs function call(s). Test compares to ground truth.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="bfcl"
LABEL dock.benchmark.description="Berkeley Function Calling Leaderboard"
LABEL dock.benchmark.tasks="2000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/ShishirPatil/gorilla"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/bfcl"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bfcl"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2406.06840"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch task and answer files from HuggingFace
RUN mkdir -p /tmp/bfcl/tasks /tmp/bfcl/answers
RUN for cat in simple multiple parallel; do \
      curl -sL "https://huggingface.co/datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard/resolve/main/BFCL_v3_${cat}.json" \
        -o /tmp/bfcl/tasks/${cat}.json && \
      curl -sL "https://huggingface.co/datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard/resolve/main/possible_answer/BFCL_v3_${cat}.json" \
        -o /tmp/bfcl/answers/${cat}.json; \
    done

# Extract to flat files
RUN python3 <<'PYEOF'
import json, os
idx = 0
for cat in ['simple', 'multiple', 'parallel']:
    tasks = [json.loads(l) for l in open(f'/tmp/bfcl/tasks/{cat}.json')]
    answers = [json.loads(l) for l in open(f'/tmp/bfcl/answers/{cat}.json')]
    answer_map = {a['id']: a for a in answers}
    for t in tasks:
        os.makedirs(f'/tasks/{idx}', exist_ok=True)
        open(f'/tasks/{idx}/id.txt','w').write(t['id'])
        # Build problem: function schemas + conversation
        funcs = json.dumps(t.get('function', []), indent=2)
        # question is [[{role, content}, ...]] — flatten
        q = t.get('question', [[]])
        msgs_list = q[0] if q and isinstance(q[0], list) else q
        msgs = '\n'.join(f"{m['role']}: {m['content']}" for m in msgs_list if isinstance(m, dict))
        open(f'/tasks/{idx}/problem.txt','w').write(
            f"Given these function definitions:\n{funcs}\n\n{msgs}")
        # Ground truth answer
        ans = answer_map.get(t['id'], {})
        open(f'/tasks/{idx}/answer.txt','w').write(
            json.dumps(ans.get('ground_truth', [])))
        idx += 1
print(f'Extracted {idx} tasks')
PYEOF
RUN rm -rf /tmp/bfcl
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=bfcl

# Test: exact match on agent stdout (simplified — full BFCL uses AST comparison)
COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Call the correct function(s) based on the conversation. Print ONLY the function call(s) as JSON, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
