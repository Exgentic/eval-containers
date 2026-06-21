# Build gosu FROM SOURCE with a current Go toolchain. The upstream release
# binaries lag a Go patch behind, so their bundled stdlib carries CVEs (incl.
# CRITICAL CVE-2025-68121); rebuilding with go1.26.4 clears them. Native build
# (no QEMU): one arch per invocation, GOARCH follows the builder — CI buildx
# amd64, local classic podman arm64, native-per-arch runners all just work, with
# no $TARGETARCH (BuildKit-only, empty under DOCKER_BUILDKIT=0).
#
# gosu is the drop-privileges helper the eval entrypoint uses to switch
# root → agent in ALL modes; it ships in the lean eval base. (process-compose,
# the single-container orchestrator, is a separate base — core/process-compose.)
FROM golang:1.26.4-alpine AS gobuild
ARG GOSU_VERSION=1.19
ENV CGO_ENABLED=0 GOOS=linux
RUN go install "github.com/tianon/gosu@${GOSU_VERSION}" \
 && mkdir -p /out \
 && cp /go/bin/gosu /out/gosu

FROM alpine:3.20 AS source

LABEL eval.type="core-base"
LABEL eval.base.kind="gosu"

# apk upgrade so this image's own alpine packages stay patched (it is gated).
RUN apk upgrade --no-cache && mkdir -p /bundle/bin

COPY --from=gobuild /out/gosu /bundle/bin/gosu
