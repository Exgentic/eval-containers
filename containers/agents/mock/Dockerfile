# syntax=docker/dockerfile:1
# mock — a deterministic agent, for testing the eval machinery rather than a model.
#
# Every other agent's output depends on a model, so a test that runs one can only
# assert that *something* happened. This one writes the same answer every time,
# which is what lets a test assert the whole output contract — task/agent/model
# result.json, the agent's own logs, the exit code — and notice when a piece of it
# stops being written.
#
# It makes no network call on purpose: the gateway and trace paths have their own
# coverage (models/replay, tests/run/replay), and a mock that reached for a model
# would stop being deterministic. Pure POSIX shell, so install.sh installs
# nothing and works on any benchmark base (agents/RULES.md 11) — the combination
# image is the benchmark base plus a few kilobytes.
FROM busybox:1.37
ARG AGENT_VERSION=1.0.0
LABEL eval.type="agent"
LABEL eval.agent.name="mock"
LABEL eval.agent.description="mock: writes a fixed answer, calls nothing — a deterministic carrier for testing the eval machinery"
LABEL eval.agent.runtime="sh"
LABEL eval.agent.url="https://github.com/Exgentic/eval-containers"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=mock

RUN mkdir -p /opt/agent && echo "${AGENT_VERSION}" > /opt/agent/VERSION \
 && printf '#!/bin/sh\n# Nothing to install: the agent is /run.sh.\nexit 0\n' > /opt/agent/install.sh \
 && chmod +x /opt/agent/install.sh

# The contract a runner expects: read $TASK, work, leave the answer on stdout.
# Both streams are written to, because the harness captures them separately and a
# test that only ever sees stdout cannot tell that stderr was dropped.
# One GET at the configured endpoint, with whatever HTTP client the base image
# happens to carry (agents/RULES.md 11 — assume no runtime). It is not a model
# call and does not pretend to be: it answers the one question a real agent asks
# first, whether OPENAI_API_BASE points somewhere this container can reach, and
# reports it on stdout so a smoke test can see the answer rather than infer it.
RUN printf '%s\n' \
    '#!/bin/sh' \
    'set -eu' \
    'base="${OPENAI_BASE_URL:-${OPENAI_API_BASE:-}}"' \
    'reach() {' \
    '  [ -n "$base" ] || return 1' \
    '  if command -v curl >/dev/null; then curl -sfm 5 "$base/v1/models" >/dev/null' \
    '  elif command -v wget >/dev/null; then wget -qT 5 -O- "$base/v1/models" >/dev/null' \
    '  elif command -v python3 >/dev/null; then' \
    '    python3 -c "import sys,urllib.request;urllib.request.urlopen(sys.argv[1],timeout=5)" "$base/v1/models"' \
    '  else return 2; fi' \
    '}' \
    'if reach; then gw=up; elif [ $? = 2 ]; then gw=unprobed; else gw=down; fi' \
    'echo "mock agent, task: ${EVAL_TASK_ID:-?}, gateway: $gw at ${base:-<unset>}" >&2' \
    'echo "gateway:$gw"' \
    'echo "${EXPECTED_ANSWER:-OK}"' \
    > /run.sh && chmod +x /run.sh

ENTRYPOINT ["/run.sh"]
