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
RUN printf '%s\n' \
    '#!/bin/sh' \
    'set -eu' \
    'echo "mock agent, task: ${EVAL_TASK_ID:-?}" >&2' \
    'echo "${TASK:-<no task>}"' \
    'echo "${EXPECTED_ANSWER:-OK}"' \
    > /run.sh && chmod +x /run.sh

ENTRYPOINT ["/run.sh"]
