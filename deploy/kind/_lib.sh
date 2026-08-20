# shellcheck shell=bash
# shellcheck disable=SC2034  # CLUSTER_DEFAULT/REGISTRY_DEFAULT/OUTPUT_HOSTPATH/REPO_DIR are read by the scripts that source this
# deploy/kind/_lib.sh — shared defaults + kind image-load helpers, sourced by the scripts.

CLUSTER_DEFAULT="eval"                 # kind cluster name (kubectl context is kind-<name>)
REGISTRY_DEFAULT="ghcr.io/exgentic"    # nested-path refs the chart composes by default (no flatImages)
OUTPUT_HOSTPATH="/eval-output"         # node-local output dir (values.yaml: outputVolume.hostPath.path)
# This lib lives at deploy/kind/, so the repo root is two levels up.
REPO_DIR="${REPO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# The container name kind gives a single control-plane node.
kind_node() { echo "$1-control-plane"; }   # arg: cluster name

# The three Job image refs for a (benchmark, agent, gatewayImage[, task]) tuple —
# nested ghcr paths, tag latest, matching the chart's _helpers.tpl composition
# (eval.runnerImage / eval.gatewayImage / eval.otelImage) exactly. Per-task swaps
# the runner name to evals/<b>-<task>--<a>. Prints "runner gateway otel".
job_refs() {  # args: <registry> <benchmark> <agent> <gatewayImage> [<task>]
  local reg="$1" b="$2" a="$3" gw="$4" task="${5:-}" runner
  if [[ -n "$task" ]]; then runner="$reg/evals/${b}-${task}--${a}:latest"
  else runner="$reg/evals/${b}--${a}:latest"; fi
  echo "$runner" "$reg/models/${gw}:latest" "$reg/core/otel:latest"
}

# Load one image ref into the cluster's containerd — but only when its content
# differs from what the node already caches. The chart runs imagePullPolicy:
# IfNotPresent on :latest tags, so once a :latest is cached in the node the node
# reuses it forever; a rebuilt image is served only after the stale copy is
# evicted. Reloading (kind load streams the whole tarball) is expensive, so skip
# it when the host image ID equals the node's cached image ID. A differing image
# ALWAYS reloads — the only thing skipped is a byte-identical cache; if the two
# id spaces ever fail to line up, the compare mismatches and it reloads anyway
# (degrades to always-reload, never to serving stale bits).
kind_reload() {  # args: <cluster> <ref> [force]
  local cluster="$1" ref="$2" force="${3:-}" node host_id node_id
  node="$(kind_node "$cluster")"
  host_id="$(docker image inspect "$ref" --format '{{.Id}}' 2>/dev/null || true)"
  [[ -z "$host_id" ]] && { echo "[run] warn: $ref not present on host — skipping load" >&2; return; }
  # Read the node's cached image id via containerd's crictl. The ref MUST precede
  # --output (crictl parses a flag placed before the image as a second image name
  # and exits fatal — crictl 1.33), and `--output go-template` / `.status.id` is
  # not portable across crictl builds. `--output json` + a grep for the first
  # `"id"` field is stable: it prints `"id": "sha256:…"`, matching docker's .Id.
  node_id="$(docker exec "$node" crictl inspecti --output json "$ref" 2>/dev/null \
             | grep -m1 '"id"' | sed -E 's/.*"id": *"([^"]+)".*/\1/' || true)"
  if [[ -z "$force" && -n "$node_id" && "$host_id" == "$node_id" ]]; then
    echo "[run] up-to-date, skip reload: $ref"
    return
  fi
  echo "[run] reload $ref"
  docker exec "$node" crictl rmi "$ref" >/dev/null 2>&1 || true
  kind load docker-image "$ref" --name "$cluster"
}
