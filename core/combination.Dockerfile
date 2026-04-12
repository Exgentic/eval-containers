ARG BENCHMARK_IMAGE
ARG AGENT_IMAGE

FROM ${BENCHMARK_IMAGE}

COPY --from=${AGENT_IMAGE} /opt/agent/install.sh /tmp/agent-install.sh
COPY --from=${AGENT_IMAGE} /opt/agent/ /opt/agent/
RUN bash /tmp/agent-install.sh && rm /tmp/agent-install.sh
