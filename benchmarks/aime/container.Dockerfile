# syntax=docker/dockerfile:1
# AIME — single-container eval image. The deployment artifact for "single
# mode": one container, all 5 units inside (otelcol → gateway → agent →
# verifier → result) orchestrated by process-compose, exits cleanly after
# the result writer finishes.
#
# This file pins the benchmark's build args; the actual recipe is the
# universal `core/combination.Dockerfile`. Building from this file is
# equivalent to invoking the universal recipe with these args:
#
#   docker build -f core/combination.Dockerfile \
#     --build-arg BENCHMARK_IMAGE=quay.io/eval-containers/benchmarks/aime:latest \
#     --build-arg AGENT_IMAGE=quay.io/eval-containers/agents/claude-code:latest \
#     --build-arg AGENT_VERSION=2.1.0 \
#     --build-arg MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--bifrost:latest \
#     -t evals/aime--claude-code .
#
# (Dockerfile has no `INCLUDE` directive, so this file pins via FROM. To
# rebuild the eval image from source, use the universal recipe above.
# This file is the registry-image lock + the build-args spec for tooling
# that scans benchmark dirs to discover the canonical eval-image pin.)
#
# Run:
#   docker run --rm \
#     -e EVAL_MODEL=openai/azure/gpt-5.4 \
#     -e OPENAI_API_KEY=... -e OPENAI_API_BASE=... \
#     -e EVAL_TASK_ID=0 \
#     -v output:/output \
#     <image-built-from-this-file>

ARG BENCHMARK_IMAGE=quay.io/eval-containers/benchmarks/aime:latest
ARG AGENT_IMAGE=quay.io/eval-containers/agents/claude-code:latest
ARG AGENT_VERSION=2.1.0
ARG MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--bifrost:latest

# Pin to the canonical pre-built eval image. Building this file pulls
# and re-tags; for a from-source rebuild use core/combination.Dockerfile
# with the build args above.
FROM quay.io/eval-containers/evals/aime--claude-code:latest
