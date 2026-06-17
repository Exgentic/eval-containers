FROM --platform=linux/amd64 alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="runtime-bundle"

# Versions bumped for CVE remediation — newer upstream releases are built
# with a newer Go toolchain, clearing the bundled Go stdlib + module CVEs:
#   gosu            1.17 -> 1.19    (built on Go 1.24.x; clears stdlib v1.18.2 CVEs)
#   process-compose 1.40.1 -> 1.116.0 (newest; clears stdlib v1.22.2 + x/crypto CVEs)
ARG GOSU_VERSION=1.19
ARG PROCESS_COMPOSE_VERSION=1.116.0

# `apk upgrade` first so the alpine base picks up any patched OS packages
# before the rest of the layers are built on top of it.
RUN apk upgrade --no-cache \
 && apk add --no-cache curl ca-certificates \
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
