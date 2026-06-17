# Build gosu + process-compose FROM SOURCE with a current Go toolchain. The
# upstream release binaries lag a Go patch behind, so their bundled stdlib
# carries CVEs (incl. CRITICAL CVE-2025-68121); rebuilding with go1.26.4 clears
# them. Cross-compiled on the native $BUILDPLATFORM (no QEMU) to the target arch.
FROM --platform=$BUILDPLATFORM golang:1.26.4-alpine AS gobuild
ARG GOSU_VERSION=1.19
ARG PROCESS_COMPOSE_VERSION=v1.116.0
ARG TARGETARCH
ENV CGO_ENABLED=0 GOOS=linux GOARCH=$TARGETARCH
# go install drops cross-built binaries under /go/bin/<goos>_<goarch>/ but native
# builds straight into /go/bin/ — normalize both into /out so the COPY is stable
# whether the runner is amd64 (CI, native) or arm64 (local, cross).
RUN go install "github.com/tianon/gosu@${GOSU_VERSION}" \
 && go install "github.com/f1bonacc1/process-compose@${PROCESS_COMPOSE_VERSION}" \
 && mkdir -p /out \
 && find /go/bin -name gosu -type f -exec cp {} /out/gosu \; \
 && find /go/bin -name process-compose -type f -exec cp {} /out/process-compose \;

FROM alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="runtime-bundle"

# apk upgrade so this image's own alpine packages stay patched (it is gated).
RUN apk upgrade --no-cache && mkdir -p /bundle/bin

# gosu: drop-privileges helper the eval entrypoint uses to switch root → agent.
# process-compose: in-container orchestrator for single-mode (otelcol + gateway
# + runner + verifier in one container). Consumers COPY /bundle/bin.
COPY --from=gobuild /out/gosu /bundle/bin/gosu
COPY --from=gobuild /out/process-compose /bundle/bin/process-compose
