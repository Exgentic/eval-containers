# TheAgentCompany (CMU)
# 175 long-horizon professional workplace tasks across software engineering,
# project management, HR, finance, admin, etc. The full benchmark requires a
# self-hosted suite of services (GitLab, RocketChat, OwnCloud, Plane) plus
# per-task Docker evaluators; we package only the task instructions and let
# upstream tooling perform the actual checkpoint grading. Reward = -1.
#
# Data: github TheAgentCompany/TheAgentCompany (workspaces/tasks/*/task.md)

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="agentcompany"
LABEL dock.benchmark.description="TheAgentCompany - long-horizon professional workplace tasks for AI agents"
LABEL dock.benchmark.tasks="175"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="98b68ef82a47690c316f42fddb05baafaab56851"
LABEL dock.benchmark.url="https://github.com/TheAgentCompany/TheAgentCompany"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2412.14161"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch the repo tree, then download each task.md to /tasks/<idx>/problem.txt.
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "98b68ef82a47690c316f42fddb05baafaab56851"
TREE_URL = f"https://api.github.com/repos/TheAgentCompany/TheAgentCompany/git/trees/{SHA}?recursive=1"
RAW = f"https://raw.githubusercontent.com/TheAgentCompany/TheAgentCompany/{SHA}/"

tree = json.loads(urllib.request.urlopen(TREE_URL).read())
paths = sorted(
    x["path"] for x in tree["tree"]
    if x["path"].startswith("workspaces/tasks/") and x["path"].endswith("/task.md")
)

for idx, path in enumerate(paths):
    name = path.split("/")[2]
    body = urllib.request.urlopen(RAW + path).read().decode("utf-8")
    d = f"/tasks/{idx}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(name)
    open(f"{d}/problem.txt", "w").write(body)
print(f"Extracted {len(paths)} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=agentcompany

# Test: externally graded by upstream per-task evaluator.py against the hosted
# services. Capture the agent's final response and write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_response.txt 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are an AI assistant working at a simulated tech company. Read the task brief below and respond with the steps you would take and any final deliverables (messages to send, files to create, answers to compute).

$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
