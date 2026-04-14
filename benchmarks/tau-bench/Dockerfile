# TAU-bench (Tool-Agent-User Benchmark — Sierra Research)
# 115 retail + 50 airline test tasks. Data: github.com/sierra-research/tau-bench
# Interactive benchmark: agent uses domain tools guided by policy documents.
# Evaluation is state-based (database + output checks), requires the TAU-bench runtime.
# Externally graded: test writes -1.

FROM python:3.12-slim

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

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Install tau-bench at pinned commit and extract tasks + policy docs
RUN pip install --no-cache-dir git+https://github.com/sierra-research/tau-bench.git@59a200c6d575d595120f1cb70fea53cef0632f6b

RUN python3 <<'PYEOF'
import json, os

# Extract test tasks from both domains
from tau_bench.envs.retail.tasks_test import TASKS_TEST as retail_tasks
from tau_bench.envs.airline.tasks_test import TASKS as airline_tasks

idx = 0

# --- Retail tasks ---
from tau_bench.envs.retail.wiki import WIKI as retail_wiki

for t in retail_tasks:
    os.makedirs(f'/tasks/{idx}', exist_ok=True)
    task_id = f'retail-{t.user_id}'
    with open(f'/tasks/{idx}/id.txt', 'w') as f:
        f.write(task_id)
    with open(f'/tasks/{idx}/domain.txt', 'w') as f:
        f.write('retail')
    with open(f'/tasks/{idx}/user_id.txt', 'w') as f:
        f.write(t.user_id)
    with open(f'/tasks/{idx}/instruction.txt', 'w') as f:
        f.write(t.instruction)
    # Store expected actions and outputs for reference (grading needs full runtime)
    with open(f'/tasks/{idx}/actions.json', 'w') as f:
        json.dump([{'name': a.name, 'kwargs': a.kwargs} for a in t.actions], f)
    with open(f'/tasks/{idx}/outputs.json', 'w') as f:
        json.dump(t.outputs, f)
    # Build agent-facing problem description
    with open(f'/tasks/{idx}/problem.txt', 'w') as f:
        f.write(f'Domain: retail\n\n')
        f.write(f'You are a retail customer service agent. Follow the policy below.\n\n')
        f.write(f'=== POLICY ===\n{retail_wiki}\n=== END POLICY ===\n\n')
        f.write(f'The customer (user_id: {t.user_id}) has the following request:\n{t.instruction}\n')
    idx += 1

print(f'Extracted {len(retail_tasks)} retail tasks')

# --- Airline tasks ---
from tau_bench.envs.airline.wiki import WIKI as airline_wiki

for t in airline_tasks:
    os.makedirs(f'/tasks/{idx}', exist_ok=True)
    task_id = f'airline-{t.user_id}'
    with open(f'/tasks/{idx}/id.txt', 'w') as f:
        f.write(task_id)
    with open(f'/tasks/{idx}/domain.txt', 'w') as f:
        f.write('airline')
    with open(f'/tasks/{idx}/user_id.txt', 'w') as f:
        f.write(t.user_id)
    with open(f'/tasks/{idx}/instruction.txt', 'w') as f:
        f.write(t.instruction)
    with open(f'/tasks/{idx}/actions.json', 'w') as f:
        json.dump([{'name': a.name, 'kwargs': a.kwargs} for a in t.actions], f)
    with open(f'/tasks/{idx}/outputs.json', 'w') as f:
        json.dump(t.outputs, f)
    with open(f'/tasks/{idx}/problem.txt', 'w') as f:
        f.write(f'Domain: airline\n\n')
        f.write(f'You are an airline customer service agent. Follow the policy below.\n\n')
        f.write(f'=== POLICY ===\n{airline_wiki}\n=== END POLICY ===\n\n')
        f.write(f'The customer (user_id: {t.user_id}) has the following request:\n{t.instruction}\n')
    idx += 1

print(f'Extracted {len(airline_tasks)} airline tasks')
print(f'Total: {idx} tasks')
PYEOF

RUN chmod -R 600 /tasks

# Add bridge-mode scripts (pass-through agent + tau-bench runner)
COPY agent.py /app/agent.py
COPY run_tau_bench.py /app/run_tau_bench.py

WORKDIR /app
ENV BENCHMARK=tau-bench

# Test: TAU-bench evaluation is state-based (database checks + output verification)
# and requires the full TAU-bench runtime with a user simulator. Externally graded.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
