FROM --platform=linux/amd64 alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="runtime-bundle"

ARG GOSU_VERSION=1.17
ARG PROCESS_COMPOSE_VERSION=1.40.1

RUN apk add --no-cache curl ca-certificates \
 && mkdir -p /bundle/bin

# gosu: drop-privileges helper used by the eval image entrypoint to switch
# from root → agent uid before launching the agent.
RUN curl -fL -o /bundle/bin/gosu \
      "https://github.com/tianon/gosu/releases/download/${GOSU_VERSION}/gosu-amd64" \
 && chmod +x /bundle/bin/gosu

# process-compose: in-container process orchestrator for single-mode
# (otelcol + gateway + runner + verifier in one container).
RUN curl -fL -o /tmp/pc.tar.gz \
      "https://github.com/F1bonacc1/process-compose/releases/download/v${PROCESS_COMPOSE_VERSION}/process-compose_linux_amd64.tar.gz" \
 && tar -xzf /tmp/pc.tar.gz -C /bundle/bin process-compose \
 && rm /tmp/pc.tar.gz \
 && chmod +x /bundle/bin/process-compose
