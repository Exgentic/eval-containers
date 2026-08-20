#!/usr/bin/env bash
# create.sh — provision the local kind cluster and the eval-secrets Secret the
# gateway reads. Split out of run.sh so cluster lifecycle (create once) is
# separate from eval submission (run many). run.sh errors if the cluster is
# absent and points here.
#
# The Secret is built from the environment: OPENAI_API_KEY (required) and
# OPENAI_API_BASE (required) — the same two keys the chart's job.yaml mounts
# (secretKeyRef eval-secrets). Export them, or source a .env, before running.
#
# EVAL_UPSTREAM_CA (optional): path to a PEM of extra CA cert(s) to trust when the
# upstream serves a private-CA TLS cert (e.g. an IBM-internal endpoint whose root
# is in your host keychain but not a pod's public trust store). Stored as the
# eval-upstream-ca ConfigMap; run.sh mounts it into the gateway and points
# SSL_CERT_FILE at it. The CA lives only on the cluster — never baked into an image.
#
#   OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-endpoint ./deploy/kind/create.sh
#   ./deploy/kind/create.sh --cluster eval --namespace my-ns
#   EVAL_UPSTREAM_CA=./ibm-ca.pem OPENAI_API_KEY=... OPENAI_API_BASE=... ./deploy/kind/create.sh
#
# --output-dir <host-path> bind-mounts a directory on your machine into the node
# at the /eval-output hostPath (via a kind `extraMounts` config), so eval results
# land on your host filesystem instead of only inside the node container. run.sh's
# default --output-path (/eval-output) then writes straight into that host dir.
# Defaults to ./eval-output in the current directory (created if absent).
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

usage() { cat <<'EOF'
create.sh — provision the local kind cluster and the eval-secrets Secret.

Usage:
  OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-endpoint ./deploy/kind/create.sh [flags]

Environment (required — the gateway reads these via the eval-secrets Secret):
  OPENAI_API_KEY    the gateway's upstream API key
  OPENAI_API_BASE   the gateway's upstream base URL, reachable from your machine

Environment (optional):
  EVAL_UPSTREAM_CA  path to a PEM of extra CA cert(s) to trust, for an upstream
                    served behind a private CA (e.g. an IBM-internal endpoint).
                    Stored as the eval-upstream-ca ConfigMap; run.sh mounts it into
                    the gateway and sets SSL_CERT_FILE. Never baked into an image.

Flags:
  --cluster <name>    kind cluster name (default: eval)
  --namespace <ns>    namespace for the Secret (default: the context's default)
  --output-dir <p>    bind-mount this host dir into the node at /eval-output, so
                      eval results land on your host filesystem (not just inside
                      the node). Default: ./eval-output (created if absent). Only
                      applies when the cluster is created.
  --recreate          delete an existing cluster of this name first, then create it fresh
  --dry-run           print the actions without creating anything
  --help              show this help and exit

Idempotent: re-running skips an existing cluster and refreshes the Secret in
place. Pass --recreate to tear an existing cluster down and rebuild it instead.
Note that --output-dir takes effect only when the cluster is created — an
existing cluster's mounts can't change, so pair it with --recreate to remount.
EOF
}

CLUSTER="$CLUSTER_DEFAULT" NAMESPACE="" OUTPUT_DIR="eval-output" DRY_RUN=false RECREATE=false
while [[ $# -gt 0 ]]; do case "$1" in
  --cluster) CLUSTER="$2"; shift 2;; --namespace) NAMESPACE="$2"; shift 2;;
  --output-dir) OUTPUT_DIR="$2"; shift 2;;
  --recreate) RECREATE=true; shift;;
  --dry-run) DRY_RUN=true; shift;;
  --help|-h) usage; exit 0;;
  *) echo "Unknown argument: $1" >&2; usage >&2; exit 1;;
esac; done
log() { echo "[create] $*"; }

KCTX="kind-$CLUSTER"   # kind's kubectl context naming convention
kube() { command kubectl --context "$KCTX" ${NAMESPACE:+-n "$NAMESPACE"} "$@"; }

# Validate credentials up front, before creating the cluster — a missing key
# should abort with no side effects, not leave a cluster and no Secret behind.
# The gateway reads OPENAI_API_KEY + OPENAI_API_BASE via secretKeyRef (the
# chart's job.yaml).
: "${OPENAI_API_KEY:?set OPENAI_API_KEY in the environment (the gateway upstream key)}"
: "${OPENAI_API_BASE:?set OPENAI_API_BASE in the environment (the gateway upstream, reachable from your machine)}"

# --output-dir bind-mounts a host dir into the node at $OUTPUT_HOSTPATH via a kind
# `extraMounts` config, so results written to /eval-output land on the host. The
# dir is created if absent (the default ./eval-output won't exist on a first run).
create_cluster() {  # renders the extraMounts config, then creates the cluster
  mkdir -p "$OUTPUT_DIR"
  local abs; abs="$(cd "$OUTPUT_DIR" && pwd)"
  local cfg; cfg="$(mktemp)"; trap 'rm -f "$cfg"' RETURN
  cat >"$cfg" <<YAML
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    extraMounts:
      - hostPath: $abs
        containerPath: $OUTPUT_HOSTPATH
YAML
  log "mounting host dir $abs → node $OUTPUT_HOSTPATH"
  kind create cluster --name "$CLUSTER" --config "$cfg"
}

# ── 1. Cluster (idempotent: create only if absent; --recreate rebuilds) ───────
if kind get clusters 2>/dev/null | grep -qx "$CLUSTER"; then
  if $RECREATE; then
    if $DRY_RUN; then log "[dry-run] kind delete cluster --name $CLUSTER; kind create cluster --name $CLUSTER (mount $OUTPUT_DIR → $OUTPUT_HOSTPATH)"
    else
      log "=== recreate kind cluster $CLUSTER (delete + create) ==="
      kind delete cluster --name "$CLUSTER"
      create_cluster
    fi
  else
    log "cluster '$CLUSTER' already exists (pass --recreate to rebuild it)"
    log "note: the output mount is fixed at cluster creation; pass --recreate to remount --output-dir"
  fi
else
  if $DRY_RUN; then log "[dry-run] kind create cluster --name $CLUSTER (mount $OUTPUT_DIR → $OUTPUT_HOSTPATH)"
  else log "=== create kind cluster $CLUSTER ==="; create_cluster; fi
fi

# ── 2. eval-secrets Secret from the environment ───────────────────────────────
# `apply` (not `create`) via a client-side render so a re-run refreshes rotated
# credentials in place rather than erroring on an existing Secret.
if $DRY_RUN; then
  log "[dry-run] apply Secret eval-secrets (OPENAI_API_KEY, OPENAI_API_BASE) in ${NAMESPACE:-default}"
else
  log "=== apply Secret eval-secrets in ${NAMESPACE:-default} ==="
  kube create secret generic eval-secrets \
    --from-literal=OPENAI_API_KEY="$OPENAI_API_KEY" \
    --from-literal=OPENAI_API_BASE="$OPENAI_API_BASE" \
    --dry-run=client -o yaml | kube apply -f -
fi

# ── 3. eval-upstream-ca ConfigMap (optional) ──────────────────────────────────
# When EVAL_UPSTREAM_CA points at a PEM, store it as a ConfigMap so run.sh can
# mount it into the gateway (SSL_CERT_FILE) for a private-CA upstream. The Go
# gateway appends SSL_CERT_FILE to the system pool on Linux, so this file need
# hold only the extra CA(s) — public roots are retained. Default-off: unset → no
# ConfigMap and the gateway trusts only public roots, exactly as before.
if [[ -n "${EVAL_UPSTREAM_CA:-}" ]]; then
  [[ -f "$EVAL_UPSTREAM_CA" ]] || { echo "error: EVAL_UPSTREAM_CA not found: $EVAL_UPSTREAM_CA" >&2; exit 1; }
  grep -q "BEGIN CERTIFICATE" "$EVAL_UPSTREAM_CA" || { echo "error: EVAL_UPSTREAM_CA has no PEM certificate: $EVAL_UPSTREAM_CA" >&2; exit 1; }
  # A CA bundle is public material; a private key here would be a leak — refuse it.
  if grep -q "PRIVATE KEY" "$EVAL_UPSTREAM_CA"; then
    echo "error: EVAL_UPSTREAM_CA contains a PRIVATE KEY — pass CA certs only" >&2; exit 1
  fi
  if $DRY_RUN; then
    log "[dry-run] apply ConfigMap eval-upstream-ca from $EVAL_UPSTREAM_CA in ${NAMESPACE:-default}"
  else
    log "=== apply ConfigMap eval-upstream-ca (from $EVAL_UPSTREAM_CA) in ${NAMESPACE:-default} ==="
    kube create configmap eval-upstream-ca \
      --from-file=ca.pem="$EVAL_UPSTREAM_CA" \
      --dry-run=client -o yaml | kube apply -f -
  fi
fi

log "ready. submit an eval with: ./deploy/kind/run.sh --benchmark <b> --agent <a> --model <m> --task 0 --watch"
