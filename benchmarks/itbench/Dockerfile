# ITBench (ITBench-Lite)
# IT automation benchmark for SRE, CISO, and FinOps scenarios.
# Data: HuggingFace ibm-research/ITBench-Lite.
#
# NOTE: This is a PLACEHOLDER skeleton. The real ITBench evaluation requires
# a live Kubernetes cluster, observability stack, databases, and web-app
# sidecars to exercise the agent against real IT systems. These sidecars are
# out of scope for this first pass. Tasks are materialised as static problem
# descriptions; test writes -1 (externally graded) until the sidecar harness
# lands.
#
# FUTURE: extend compose.yaml with a multi-service stack (k8s-in-docker,
# prometheus, loki, opa, test app) and wire scenario-specific ground-truth
# verifiers from the upstream eval.py into test.sh.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="itbench"
LABEL eval.benchmark.description="ITBench - IT automation across SRE, CISO, FinOps scenarios (skeleton)"
LABEL eval.benchmark.tasks="10"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="3c36485964b059621d439343009ec0e7b4d2354b"
LABEL eval.benchmark.url="https://github.com/itbench-hub/ITBench"
LABEL eval.benchmark.paper="https://research.ibm.com/publications/developing-ai-agents-for-it-automation-tasks-with-itbench"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="3c36485964b059621d439343009ec0e7b4d2354b"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
# For the first pass we ingest only CISO k8s-opa-static scenarios from
# ITBench-Lite (10 tasks). FinOps and SRE scenarios require active
# cluster/app telemetry and are deferred until the sidecar harness exists.
RUN python3 <<'PYEOF'
import urllib.request, os, json
REV = "3c36485964b059621d439343009ec0e7b4d2354b"
api = f'https://huggingface.co/api/datasets/ibm-research/ITBench-Lite/tree/{REV}/snapshots/ciso/v0.1?recursive=false'
with urllib.request.urlopen(api) as r:
    entries = json.load(r)
scenarios = sorted(e['path'].split('/')[-1] for e in entries if e.get('type') == 'directory')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for s in scenarios:
        url = f'https://huggingface.co/datasets/ibm-research/ITBench-Lite/resolve/{REV}/snapshots/ciso/v0.1/{s}/problem.md'
        try:
            with urllib.request.urlopen(url) as r:
                problem = r.read().decode('utf-8')
        except Exception as e:
            print(f'skip {s}: {e}')
            continue
        row = {
            'id': s,
            'problem': problem,
            'domain': 'ciso',
            'scenario': s,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=itbench

# ITBench evaluation replays agent remediations against a live environment
# (K8s cluster with injected incidents, OPA policies, Prometheus targets,
# etc). The verifier is scenario-specific and lives in the upstream eval.py.
# Externally graded per compose/RULES.md rule 16 until the harness ships.
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
