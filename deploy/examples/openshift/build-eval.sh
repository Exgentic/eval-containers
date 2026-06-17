#!/usr/bin/env bash
# Build a full benchmark × agent eval on an OpenShift cluster, in dependency
# order, using the CLI's OpenShift backend (`eval-containers build --builder oc`)
# for each artifact. The CLI does each native build (BuildConfig + oc start-build,
# deps resolved from the internal registry via the parameterized ${REGISTRY}
# FROMs); this script is just the dependency-ordered loop over it — the glue
# that, per doctrine, stays out of the CLI binary (src/RULES.md principle 3).
#
# Prerequisites (see README.md):
#   - `oc login` to the cluster, `oc project <namespace>`
#   - the namespace has `anyuid-sa` + `eval-secrets` (deploy/openshift-service-account.yaml)
#   - the CORE base images already exist in the internal registry
#     (one-time bootstrap — see "Bootstrapping core bases" in README.md)
#
# Usage:  ./build-eval.sh <benchmark> <agent> [model]
set -euo pipefail

BENCHMARK=${1:?usage: build-eval.sh <benchmark> <agent> [model]}
AGENT=${2:?usage: build-eval.sh <benchmark> <agent> [model]}
MODEL=${3:-bifrost}

cd "$(dirname "$0")/../../.."   # repo root (contexts are relative to it)

# The three bases are independent (given core), the combination depends on all
# three — so: bases first, eval last. The CLI derives every imagestream name,
# context, and build-arg; this loop only encodes the ordering.
eval-containers build bench "$BENCHMARK"                          --builder oc
eval-containers build agent "$AGENT"                             --builder oc
eval-containers build model "$MODEL"                             --builder oc
eval-containers build eval  "$BENCHMARK" --agent "$AGENT" --model "$MODEL" --builder oc

echo "✓ built ${BENCHMARK}-${AGENT} on the cluster (internal registry)"
