# syntax=docker/dockerfile:1
# native — the fleet's one non-agent: the benchmark's own harness is the agent.
#
# A native-harness benchmark (benchmarks/RULES.md 12a) drives the model through
# its upstream scaffold — tau-bench's tool loop and user simulator, walle's
# response_format probe, AutomationBench's auto-bench — so there is no agent to
# install. This image exists so such a benchmark still has exactly one eval
# image, named and built like every other (evals/<benchmark>--native), instead
# of one per agent that never runs.
#
# /run.sh execs the harness the benchmark ships at /harness.sh. /opt/agent/native
# tells run-agent to launch it privileged and with the task identity — the
# verifier's position of trust, not the scrubbed agent phase — so a bundle run
# with no EVAL_AGENT set still takes that path (agents/RULES.md 23).
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
 && printf '#!/bin/sh\n# Nothing to install: the benchmark ships the harness.\nexit 0\n' > /opt/agent/install.sh \
 && chmod +x /opt/agent/install.sh \
 && : > /opt/agent/native \
 && printf '#!/bin/sh\nexec /harness.sh\n' > /run.sh && chmod +x /run.sh

ENTRYPOINT ["/run.sh"]
