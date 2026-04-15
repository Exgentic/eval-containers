ARG BENCHMARK_IMAGE
ARG AGENT_IMAGE
ARG AGENT_VERSION

FROM ${BENCHMARK_IMAGE}

ARG AGENT_IMAGE
ARG AGENT_VERSION

COPY --from=${AGENT_IMAGE} /opt/agent/install.sh /tmp/agent-install.sh
COPY --from=${AGENT_IMAGE} /opt/agent/ /opt/agent/
# Propagate the agent's pinned version into the combined image so
# core/entrypoint/dock-entrypoint.sh can record it in version.json
# (RULES.md principle 9). The CLI reads the agent image's
# LABEL dock.agent.version at build time and passes it as AGENT_VERSION.
ENV DOCK_AGENT_VERSION_DEFAULT=${AGENT_VERSION}
RUN bash /tmp/agent-install.sh && rm /tmp/agent-install.sh
