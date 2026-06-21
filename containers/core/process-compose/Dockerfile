# Build process-compose FROM SOURCE with a current Go toolchain. The upstream
# release binaries lag a Go patch behind, so their bundled stdlib carries CVEs
# (incl. CRITICAL CVE-2025-68121); rebuilding with go1.26.4 clears them. Native
# build (no QEMU): one arch per invocation, GOARCH follows the builder — CI
# buildx amd64, local classic podman arm64, native-per-arch runners all just
# work, with no $TARGETARCH (BuildKit-only, empty under DOCKER_BUILDKIT=0).
#
# process-compose is the in-container orchestrator for single-container /
# -standalone mode (otelcol + gateway + runner + verifier in one container); it
# ships ONLY in core/standalone.Dockerfile. The lean eval base needs none of it.
# (gosu, the drop-privileges helper used in ALL modes, is a separate base —
# core/gosu.) The orchestration config + scripts this binary runs
# (process-compose.yaml, run, run-agent, write-result) also live in this dir and
# are COPY'd by the combination/standalone Dockerfiles from the core context.
FROM golang:1.26.4-alpine AS gobuild
ARG PROCESS_COMPOSE_VERSION=v1.116.0
ENV CGO_ENABLED=0 GOOS=linux
RUN go install "github.com/f1bonacc1/process-compose@${PROCESS_COMPOSE_VERSION}" \
 && mkdir -p /out \
 && cp /go/bin/process-compose /out/process-compose

FROM alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="process-compose"

# apk upgrade so this image's own alpine packages stay patched (it is gated).
RUN apk upgrade --no-cache && mkdir -p /bundle/bin

COPY --from=gobuild /out/process-compose /bundle/bin/process-compose
