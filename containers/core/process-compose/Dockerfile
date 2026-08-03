# Build process-compose FROM SOURCE with a current Go toolchain. The upstream
# release binaries lag a Go patch behind, so their bundled stdlib carries CVEs
# (incl. CRITICAL CVE-2025-68121); rebuilding with go1.26.5 clears them (incl.
# CVE-2026-39822), and x/text is forced to the CVE-patched 0.39.0 (CVE-2026-56852). Native
# build (no QEMU): one arch per invocation, GOARCH follows the builder — CI
# buildx amd64, local classic podman arm64, native-per-arch runners all just
# work, with no $TARGETARCH (BuildKit-only, empty under DOCKER_BUILDKIT=0).
#
# process-compose is the in-container orchestrator for single-container /
# -standalone mode (otelcol + gateway + runner + verifier in one container); it
# ships ONLY in core/standalone.Dockerfile. The lean eval base needs none of it.
# (gosu, the drop-privileges helper used in ALL modes, is a separate base —
# core/gosu.) This dir builds ONLY the binary; the pipeline scripts it runs and
# the process-compose.yaml supervision graph live in core/runner/ (mode-agnostic
# — the same scripts drive the lean base's runner sequence with no supervisor).
FROM golang:1.26.5-alpine AS gobuild
RUN apk add --no-cache git
ARG PROCESS_COMPOSE_VERSION=v1.116.0
ENV CGO_ENABLED=0 GOOS=linux
# Build from source so x/text can be forced to the CVE-patched 0.39.0 (a plain
# `go install pkg@ver` can't override a transitive dep).
RUN git clone --depth 1 --branch "${PROCESS_COMPOSE_VERSION}" https://github.com/f1bonacc1/process-compose /src \
 && cd /src \
 && go get golang.org/x/text@v0.39.0 \
 && go mod tidy \
 && go build -o /process-compose . \
 && mkdir -p /out \
 && cp /process-compose /out/process-compose

FROM alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="process-compose"

# apk upgrade so this image's own alpine packages stay patched (it is gated).
RUN apk upgrade --no-cache && mkdir -p /bundle/bin

COPY --from=gobuild /out/process-compose /bundle/bin/process-compose
