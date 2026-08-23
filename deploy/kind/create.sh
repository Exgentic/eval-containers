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
# After provisioning, create.sh probes OPENAI_API_BASE from the cluster (GET
# /models) and prints the models the upstream offers. If it is unreachable from
# the cluster, it reports whether the host can reach it — a private CA the host
# trusts but a pod does not is the usual cause — and points at deploy/kind/README.md
# § 'Private-CA upstreams'. The probe is read-only; it writes nothing.
#
# EVAL_UPSTREAM_CA (optional): path to a PEM of extra CA cert(s) for a private-CA
# upstream. Stored as the eval-upstream-ca ConfigMap; run.sh mounts it into the
# gateway and points SSL_CERT_FILE at it. The CA lives only on the cluster —
# never baked into an image. See the README section above.
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

Upstream probe:
  After provisioning, create.sh probes OPENAI_API_BASE from the cluster (GET
  /models) and lists the models the upstream offers. If it is unreachable from
  the cluster, it reports whether the host can reach it (a private CA the host
  trusts but a pod does not is the usual cause) and points at the README
  § 'Private-CA upstreams'. Read-only — it writes nothing to the cluster.

Environment (optional):
  EVAL_UPSTREAM_CA  path to a PEM of CA cert(s) to trust, for an upstream served
                    behind a private CA. Stored as the eval-upstream-ca ConfigMap;
                    run.sh mounts it into the gateway (SSL_CERT_FILE). Never baked
                    into an image. See the README § 'Private-CA upstreams'.

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

# Validate a PEM of CA cert(s) and store it as the eval-upstream-ca ConfigMap that
# run.sh mounts into the gateway (SSL_CERT_FILE), for a private-CA upstream.
# Refuses a file that carries a private key. Arg: path to a PEM file.
store_upstream_ca() {  # arg: <pem-file>
  local pem="$1"
  [[ -f "$pem" ]] || { echo "error: CA file not found: $pem" >&2; return 1; }
  grep -q "BEGIN CERTIFICATE" "$pem" || { echo "error: no PEM certificate in: $pem" >&2; return 1; }
  # A CA bundle is public material; a private key here would be a leak — refuse it.
  if grep -q "PRIVATE KEY" "$pem"; then
    echo "error: CA file contains a PRIVATE KEY — pass CA certs only: $pem" >&2; return 1
  fi
  log "=== apply ConfigMap eval-upstream-ca (from $pem) in ${NAMESPACE:-default} ==="
  kube create configmap eval-upstream-ca \
    --from-file=ca.pem="$pem" \
    --dry-run=client -o yaml | kube apply -f -
}

# curl image for the throwaway in-cluster probe (a local dev diagnostic, not a
# released fleet image — the fleet's digest-pin rule governs containers/, not
# this). A probe pull uses the same cluster egress the probe tests, so a pull
# failure degrades to "skip", never a hang.
PROBE_IMAGE="curlimages/curl:8.11.1"

# GET {base}/models from a throwaway pod — through the same egress + trust store
# the gateway pod uses. Prints the full response body followed by a final line
# `__rc__=<n>` carrying curl's own exit code, so the caller can tell "curl failed"
# apart from "curl succeeded but the body held no model ids" (a grep on the body
# alone conflates the two). Never fails the pipeline itself.
probe_models_from_cluster() {
  # No -o /dev/null: we want the body to parse model ids from. A non-2xx body still
  # means TLS + network worked — reachability is curl's exit code, not HTTP status.
  # KEY/URL come in via --env (kept out of the command args), read inside the pod.
  kube run eval-upstream-probe --rm -i --restart=Never --quiet \
    --pod-running-timeout=40s --image="$PROBE_IMAGE" \
    --env="KEY=$OPENAI_API_KEY" --env="URL=${OPENAI_API_BASE%/}" --command -- \
    sh -c 'curl -sS --max-time 15 -H "Authorization: Bearer $KEY" "$URL/models"; rc=$?; echo; echo "__rc__=$rc"' \
    2>/dev/null
}

# Report which models the upstream offers and, if it is unreachable from the
# cluster, whether the host can reach it — then point at the README for the fix.
# Read-only: it never writes to the cluster. Skipped under --dry-run (it starts a
# throwaway pod) and never fatal — a failed probe only prints a diagnosis.
probe_and_report() {
  $DRY_RUN && { log "[dry-run] skip upstream reachability/model probe"; return 0; }

  log "probing $OPENAI_API_BASE from the cluster (GET /models) …"
  # Run the substitution where a failure is expected so set -e doesn't abort first.
  local resp="" rc=0
  resp="$(probe_models_from_cluster)" || rc=$?
  # Recover curl's real exit code from the trailer, then strip the trailer line.
  local curl_rc="$rc" body="$resp"
  if [[ "$resp" == *"__rc__="* ]]; then
    curl_rc="${resp##*__rc__=}"; curl_rc="${curl_rc%%[!0-9]*}"
    body="${resp%$'\n'__rc__=*}"
  fi

  if [[ "${curl_rc:-1}" -eq 0 ]]; then
    local models
    models="$(printf '%s' "$body" | grep -oE '"id"[[:space:]]*:[[:space:]]*"[^"]+"' \
                | sed -E 's/.*"([^"]+)"$/\1/' || true)"
    if [[ -n "$models" ]]; then
      log "upstream reachable from the cluster. available models:"
      while IFS= read -r m; do log "  - $m"; done <<<"$models"
    else
      log "upstream reachable from the cluster, but GET /models returned no model ids"
      log "(the endpoint may not be OpenAI-/models-shaped — reachability is fine)."
    fi
    return 0
  fi
  rc="$curl_rc"

  # Unreachable from the cluster. Is it the host's trust/network, or the cluster's?
  log "upstream NOT reachable from the cluster (curl exit $rc)."
  if curl -sS -o /dev/null --max-time 15 "$OPENAI_API_BASE" >/dev/null 2>&1; then
    log "…but the HOST reaches $OPENAI_API_BASE. The cluster likely lacks a private CA"
    log "the host trusts (curl 60 / x509: unknown authority). See deploy/kind/README.md"
    log "§ 'Private-CA upstreams' to install the CA into the cluster."
  else
    log "…and the HOST cannot reach it either — likely a network/DNS/endpoint issue,"
    log "not a certificate. Verify OPENAI_API_BASE and your connectivity. If it is a"
    log "private-CA endpoint, see deploy/kind/README.md § 'Private-CA upstreams'."
  fi
}

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

# ── 3. eval-upstream-ca ConfigMap (optional, explicit) ────────────────────────
# When EVAL_UPSTREAM_CA points at a PEM, store it as the eval-upstream-ca
# ConfigMap that run.sh mounts into the gateway (SSL_CERT_FILE) for a private-CA
# upstream; the Go gateway appends it to the system pool on Linux, so the file
# need hold only the extra CA(s). Default-off: unset → no ConfigMap and the
# gateway trusts only public roots, as before. See deploy/kind/README.md
# § 'Private-CA upstreams'.
if [[ -n "${EVAL_UPSTREAM_CA:-}" ]]; then
  if $DRY_RUN; then
    log "[dry-run] apply ConfigMap eval-upstream-ca from $EVAL_UPSTREAM_CA in ${NAMESPACE:-default}"
  else
    store_upstream_ca "$EVAL_UPSTREAM_CA" || exit 1
  fi
fi

# ── 4. Upstream reachability + model report (read-only diagnostic) ────────────
# Confirm the gateway's upstream is reachable from the cluster and list the models
# it offers; if it is not, say whether the host can reach it and point at the
# README. Purely informational — writes nothing to the cluster.
probe_and_report

log "ready. submit an eval with: ./deploy/kind/run.sh --benchmark <b> --agent <a> --model <m> --task 0 --watch"
