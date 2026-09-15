# syntax=docker/dockerfile:1
# native — no agent: the benchmark's own harness (/harness.sh) is the agent, so a
# native-harness benchmark still has one eval image, evals/<benchmark>--native.
# /opt/agent/native tells run-agent to launch it privileged with the task identity.
FROM busybox:1.37
ARG AGENT_VERSION=1.0.0
LABEL eval.type="agent"
LABEL eval.agent.name="native"
LABEL eval.agent.description="native: the benchmark's own harness is the agent — pairs only with a benchmark that declares eval.benchmark.agent=native"
LABEL eval.agent.runtime="sh"
LABEL eval.agent.url="https://github.com/Exgentic/eval-containers"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=native

RUN mkdir -p /opt/agent && echo "${AGENT_VERSION}" > /opt/agent/VERSION \
 && printf '#!/bin/sh\nexit 0\n' > /opt/agent/install.sh && chmod +x /opt/agent/install.sh \
 && : > /opt/agent/native \
 && printf '#!/bin/sh\nexec /harness.sh\n' > /run.sh && chmod +x /run.sh

ENTRYPOINT ["/run.sh"]
