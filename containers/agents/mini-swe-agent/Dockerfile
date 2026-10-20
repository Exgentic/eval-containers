# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=2.4.6
LABEL eval.type="agent"
LABEL eval.agent.name="mini-swe-agent"
LABEL eval.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/SWE-agent/mini-swe-agent"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=mini-swe-agent
RUN uv pip install --system --no-cache mini-swe-agent==${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
# litellm pinned: mini leaves it unbounded, and 1.106.0 imports `mcp` (not a
# dependency) on every tool-calling completion, so each model call fails.
/opt/agent/setup-uv-venv.sh /opt/mini-venv mini-swe-agent==${AGENT_VERSION} litellm==1.104.1
I

# Inlines every image file $TASK names by absolute path as a data URL, in the
# tag mini's multimodal_regex expands into an image_url part — the way SWE-agent
# runs SWE-bench Multimodal. Keyed on the file existing, never on the benchmark
# (agents/RULES.md rule 4). Writes the task and the regex as a mini config file
# (JSON is YAML), never an argument or env var: a few screenshots outgrow
# Linux's 128 KiB per-string limit. Writes nothing when the task names no image,
# so a text-only task stays a plain string.
RUN cat > /opt/agent/attach-images.py <<'P'
import base64, json, mimetypes, os, re, sys

task = os.environ["TASK"]
found = False

def attach(m):
    global found
    path = m.group(0)
    if not os.path.isfile(path):
        return path
    found = True
    data = base64.b64encode(open(path, "rb").read()).decode()
    mime = mimetypes.guess_type(path)[0]
    return (f"{path} <MSWEA_MULTIMODAL_CONTENT><CONTENT_TYPE>image_url</CONTENT_TYPE>"
            f"data:{mime};base64,{data}</MSWEA_MULTIMODAL_CONTENT>")

out = re.sub(r"/[^\s)\]\"'<>]+\.(?:png|jpe?g|gif|webp)\b", attach, task, flags=re.I)
if found:
    regex = r"(?s)<MSWEA_MULTIMODAL_CONTENT><CONTENT_TYPE>(.+?)</CONTENT_TYPE>(.+?)</MSWEA_MULTIMODAL_CONTENT>"
    with open(sys.argv[1], "w") as f:
        json.dump({"run": {"task": out}, "model": {"multimodal_regex": regex}}, f)
P

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
# Model name comes from $MODEL: run-agent scrubs the env (`env -i`) and maps
# EVAL_MODEL → MODEL, so the agent never sees EVAL_MODEL itself.
# MSWEA_CONFIGURED skips mini's first-run setup wizard.
# MSWEA_COST_TRACKING=ignore_errors + --cost-limit 0: mini's litellm cost lookup
# raises for any model name absent from litellm's cost map, and its default $3
# cap would stop the run on mini's own estimate. The gateway is the cost
# authority (EVAL_MODEL_MAX_BUDGET) and run-agent enforces the timeout.
# MSWEA_GLOBAL_CONFIG_DIR keeps mini's config + trajectory under /tmp (rule 8).
# --exit-immediately: without it mini prompts for confirmation after the agent
# submits, and nobody is there to answer.
export MSWEA_CONFIGURED=1
export MSWEA_COST_TRACKING=ignore_errors
export MSWEA_GLOBAL_CONFIG_DIR=/tmp/mini-swe-agent
# A task that names image files gets them as images, not paths: mini sends an
# image only inside its multimodal tag, and only with multimodal_regex set (off
# by default). That config file carries the task, so --task is left off; `-c`
# replaces the default config, hence mini.yaml first.
mkdir -p "$MSWEA_GLOBAL_CONFIG_DIR"
images="$MSWEA_GLOBAL_CONFIG_DIR/task-with-images.yaml"
rm -f "$images"
/opt/mini-venv/bin/python /opt/agent/attach-images.py "$images"
task=(--task "$TASK")
[ ! -f "$images" ] || task=(-c mini.yaml -c "$images")
exec /opt/mini-venv/bin/mini --model "openai/${MODEL:-default}" --yolo --exit-immediately \
  --cost-limit 0 "${task[@]}"
E

ENTRYPOINT ["/run.sh"]
