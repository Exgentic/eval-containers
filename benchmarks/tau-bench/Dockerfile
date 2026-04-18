# TAU-bench (Tool-Agent-User Benchmark - Sierra Research)
# 115 retail + 50 airline test tasks. Data: github.com/sierra-research/tau-bench
# Interactive benchmark: agent uses domain tools guided by policy documents.
# Evaluation is state-based (database + output checks), requires the TAU-bench runtime.
# Externally graded: test writes -1.

FROM quay.io/dock-eval/core/benchmark-base-external:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="tau-bench"
LABEL dock.benchmark.description="TAU-bench - Tool-Agent-User interaction (retail + airline)"
LABEL dock.benchmark.tasks="165"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="59a200c6d575d595120f1cb70fea53cef0632f6b"
LABEL dock.benchmark.url="https://github.com/sierra-research/tau-bench"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/tau2"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2406.12045"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="59a200c6d575d595120f1cb70fea53cef0632f6b"

# Install tau-bench at pinned commit and extract tasks + policy docs
RUN pip install --no-cache-dir git+https://github.com/sierra-research/tau-bench.git@59a200c6d575d595120f1cb70fea53cef0632f6b

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/.
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
ENV DOCK_BENCHMARK=tau-bench

# Test: TAU-bench evaluation is state-based (database checks + output verification)
# and requires the full TAU-bench runtime with a user simulator. Externally graded.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
