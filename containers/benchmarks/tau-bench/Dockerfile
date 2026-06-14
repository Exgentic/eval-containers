# TAU-bench (Tool-Agent-User Benchmark - Sierra Research)
# 115 retail + 50 airline test tasks. Data: github.com/sierra-research/tau-bench
# Interactive benchmark: agent uses domain tools guided by policy documents.
# Evaluation is state-based (database + output checks), requires the TAU-bench runtime.
# Externally graded: test writes -1.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="tau-bench"
LABEL eval.benchmark.description="TAU-bench - Tool-Agent-User interaction (retail + airline)"
LABEL eval.benchmark.tasks="165"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="59a200c6d575d595120f1cb70fea53cef0632f6b"
LABEL eval.benchmark.url="https://github.com/sierra-research/tau-bench"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/tau2"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.12045"

# Install tau-bench at pinned commit and extract tasks + policy docs
RUN pip install --no-cache-dir git+https://github.com/sierra-research/tau-bench.git@59a200c6d575d595120f1cb70fea53cef0632f6b

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import json, os

from tau_bench.envs.retail.tasks_test import TASKS_TEST as retail_tasks
from tau_bench.envs.airline.tasks_test import TASKS as airline_tasks
from tau_bench.envs.retail.wiki import WIKI as retail_wiki
from tau_bench.envs.airline.wiki import WIKI as airline_wiki

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for domain, wiki, tasks in (
        ('retail', retail_wiki, retail_tasks),
        ('airline', airline_wiki, airline_tasks),
    ):
        for t in tasks:
            problem = (
                f'Domain: {domain}\n\n'
                f'You are a {domain} customer service agent. Follow the policy below.\n\n'
                f'=== POLICY ===\n{wiki}\n=== END POLICY ===\n\n'
                f'The customer (user_id: {t.user_id}) has the following request:\n{t.instruction}\n'
            )
            row = {
                'id': f'{domain}-{t.user_id}',
                'domain': domain,
                'user_id': t.user_id,
                'instruction': t.instruction,
                'problem': problem,
                'actions': [{'name': a.name, 'kwargs': a.kwargs} for a in t.actions],
                'outputs': t.outputs,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Add bridge-mode scripts (pass-through agent + tau-bench runner)
COPY agent.py /app/agent.py
COPY run_tau_bench.py /app/run_tau_bench.py

WORKDIR /app
ENV EVAL_BENCHMARK=tau-bench

# Test: TAU-bench evaluation is state-based (database checks + output verification)
# and requires the full TAU-bench runtime with a user simulator. Externally graded.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
