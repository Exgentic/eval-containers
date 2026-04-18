# ACPBench
# 7 reasoning tasks x 13 domains. Data: HuggingFace ibm-research/acp_bench (MCQ split).
# Agent prints a single letter (A/B/C/D). Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="acpbench"
LABEL dock.benchmark.description="ACPBench - Action, Change, and Planning reasoning benchmark"
LABEL dock.benchmark.tasks="1040"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="05e5883d9afbcfa3bdc8f270cb345ef0b9526d4a"
LABEL dock.benchmark.url="https://ibm.github.io/ACPBench/"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2410.05669"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="05e5883d9afbcfa3bdc8f270cb345ef0b9526d4a"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
# Use the MCQ splits (boolean/generative are graded differently and deferred).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
REV = "05e5883d9afbcfa3bdc8f270cb345ef0b9526d4a"
GROUPS = [
    "acp_app_mcq", "acp_areach_mcq", "acp_just_mcq",
    "acp_land_mcq", "acp_prog_mcq", "acp_reach_mcq", "acp_val_mcq",
]
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for g in GROUPS:
        url = f'https://huggingface.co/datasets/ibm-research/acp_bench/resolve/{REV}/{g}/test.parquet'
        fp = f'/tmp/{g}.parquet'
        urllib.request.urlretrieve(url, fp)
        t = pq.read_table(fp)
        for i in range(len(t)):
            # Build problem text: context + question + choices.
            ctx = str(t['context'][i])
            q = str(t['question'][i])
            problem = f"{ctx}\n\n{q}"
            row = {
                'id': f"{g}_{i}",
                'problem': problem,
                'answer': str(t['answer'][i]).strip(),
                'group': g,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove(fp)
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=acpbench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
