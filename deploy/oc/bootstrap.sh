#!/usr/bin/env bash
# bootstrap.sh — namespace prereqs deploy/oc/README.md already assumes
# ("applied once from deploy/"), made runnable and idempotent: namespace,
# anyuid SCC, output PVC, registry pull secret (external-registry mode), and
# an eval-secrets presence check. Safe to re-run — every step is check-then-act.
#
#   ./oc/bootstrap.sh --namespace eval-containers-b --storage-class ibm-spectrum-scale-fileset
#   ./oc/bootstrap.sh --namespace eval-containers-b --storage-class <sc> \
#     --registry-mode external --registry-server icr.io --registry-username iamapikey
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

NAMESPACE="" STORAGE_CLASS="" PVC_SIZE="20Gi" REGISTRY_MODE="internal"
REGISTRY_SERVER="" REGISTRY_USERNAME="" PULL_SECRET_NAME=""
SKIP_PVC=false SKIP_SCC=false DRY_RUN=false YES=false
while [[ $# -gt 0 ]]; do case "$1" in
  --namespace) NAMESPACE="$2"; shift 2;; --storage-class) STORAGE_CLASS="$2"; shift 2;;
  --pvc-size) PVC_SIZE="$2"; shift 2;; --registry-mode) REGISTRY_MODE="$2"; shift 2;;
  --registry-server) REGISTRY_SERVER="$2"; shift 2;;
  --registry-username) REGISTRY_USERNAME="$2"; shift 2;;
  --pull-secret-name) PULL_SECRET_NAME="$2"; shift 2;;
  --skip-pvc) SKIP_PVC=true; shift;; --skip-scc) SKIP_SCC=true; shift;;
  --dry-run) DRY_RUN=true; shift;; --yes) YES=true; shift;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$NAMESPACE" || -z "$STORAGE_CLASS" ]] && {
  echo "error: --namespace and --storage-class are required" >&2; exit 1; }
[[ "$REGISTRY_MODE" == external ]] && [[ -z "$REGISTRY_SERVER" || -z "$REGISTRY_USERNAME" ]] && {
  echo "error: --registry-mode external requires --registry-server and --registry-username" >&2; exit 1; }
PULL_SECRET_NAME="${PULL_SECRET_NAME:-${REGISTRY_SERVER//./-}-pull-secret}"
log() { echo "[bootstrap] $*"; }

# ── 1. Namespace ──────────────────────────────────────────────────────────────
ensure_namespace() {
  if command oc get ns "$NAMESPACE" &>/dev/null; then
    log "namespace $NAMESPACE exists"
    return
  fi
  $DRY_RUN && { log "[dry-run] oc new-project $NAMESPACE"; return; }
  command oc new-project "$NAMESPACE" >/dev/null
  log "namespace $NAMESPACE created"
}

# ── 2. Service account + SCC (cluster-admin-adjacent — confirm) ─────────────
ensure_scc() {
  $SKIP_SCC && { log "skip SCC (--skip-scc)"; return; }
  $DRY_RUN && {
    log "[dry-run] oc apply -f deploy/openshift-service-account.yaml -n $NAMESPACE"
    command oc get rolebinding -n "$NAMESPACE" 2>/dev/null | grep -q 'scc:anyuid' \
      && log "[dry-run] scc:anyuid rolebinding already present" \
      || log "[dry-run] oc adm policy add-scc-to-user anyuid -z anyuid-sa -n $NAMESPACE"
    return
  }
  command oc apply -f "$REPO_DIR/deploy/openshift-service-account.yaml" -n "$NAMESPACE" >/dev/null
  if command oc get rolebinding -n "$NAMESPACE" 2>/dev/null | grep -q 'scc:anyuid'; then
    log "anyuid SCC already granted"
    return
  fi
  if ! $YES; then
    read -rp "[bootstrap] grant anyuid SCC to anyuid-sa in $NAMESPACE? this is the one cluster-admin-adjacent action here [y/N] " ans
    [[ "$ans" == y || "$ans" == Y ]] || { echo "error: aborted (pass --yes to skip this confirmation)" >&2; exit 1; }
  fi
  command oc adm policy add-scc-to-user anyuid -z anyuid-sa -n "$NAMESPACE"
  log "anyuid SCC granted"
}

# ── 3. Output PVC (storage class is immutable on a bound PVC — never patch) ──
ensure_pvc() {
  $SKIP_PVC && { log "skip PVC (--skip-pvc)"; return; }
  local existing
  existing=$(command oc get pvc eval-output-pvc -n "$NAMESPACE" -o jsonpath='{.spec.storageClassName}' 2>/dev/null || true)
  if [[ -n "$existing" ]]; then
    if [[ "$existing" != "$STORAGE_CLASS" ]]; then
      echo "error: eval-output-pvc already exists with storageClassName=$existing, requested $STORAGE_CLASS (storage class is immutable on a bound PVC; not patching — delete the PVC by hand first if you really want to change it)" >&2
      exit 1
    fi
    log "eval-output-pvc exists (storageClassName=$existing)"
    return
  fi
  # The committed manifest is templated via sed at apply time, never edited on disk.
  local rendered
  rendered=$(sed -e "s#namespace: exgentic-ns#namespace: $NAMESPACE#" \
                  -e "s#storageClassName: ibmc-vpc-file-retain-1000-iops#storageClassName: $STORAGE_CLASS#" \
                  -e "s#storage: 20Gi#storage: $PVC_SIZE#" \
                  "$REPO_DIR/deploy/eval-output-pvc.yaml")
  $DRY_RUN && { log "[dry-run] apply eval-output-pvc (storageClassName=$STORAGE_CLASS, size=$PVC_SIZE)"; return; }
  printf '%s\n' "$rendered" | command oc apply -n "$NAMESPACE" -f - >/dev/null
  log "eval-output-pvc created (storageClassName=$STORAGE_CLASS)"
}

# ── 4. Registry pull secret (external-registry mode only) ───────────────────
ensure_pull_secret() {
  [[ "$REGISTRY_MODE" != external ]] && { log "registry-mode=$REGISTRY_MODE, skip pull secret"; return; }
  $DRY_RUN && {
    log "[dry-run] oc create secret docker-registry $PULL_SECRET_NAME --docker-server=$REGISTRY_SERVER --docker-username=$REGISTRY_USERNAME --docker-password=<hidden> --dry-run=client -o yaml | oc apply -n $NAMESPACE -f -"
    log "[dry-run] oc secrets link anyuid-sa $PULL_SECRET_NAME --for=pull -n $NAMESPACE"
    return
  }
  # Password never appears on the command line or in shell history: env var or
  # an interactive, non-echoing prompt only.
  local password
  if [[ -n "${REGISTRY_PASSWORD:-}" ]]; then
    password="$REGISTRY_PASSWORD"
  else
    read -rsp "[bootstrap] registry password/token for $REGISTRY_USERNAME@$REGISTRY_SERVER: " password
    echo >&2
  fi
  command oc create secret docker-registry "$PULL_SECRET_NAME" \
    --docker-server="$REGISTRY_SERVER" --docker-username="$REGISTRY_USERNAME" \
    --docker-password="$password" --dry-run=client -o yaml \
    | command oc apply -n "$NAMESPACE" -f - >/dev/null
  command oc secrets link anyuid-sa "$PULL_SECRET_NAME" --for=pull -n "$NAMESPACE"
  log "pull secret $PULL_SECRET_NAME ready, linked to anyuid-sa"
}

# ── 5. eval-secrets — presence check only, never created here ───────────────
check_eval_secrets() {
  if command oc get secret eval-secrets -n "$NAMESPACE" -o name &>/dev/null; then
    log "eval-secrets found"
    return
  fi
  cat >&2 <<EOF
error: eval-secrets not found in namespace $NAMESPACE. Create it by hand
(key material is never handled by this script):

  oc create secret generic eval-secrets -n $NAMESPACE \\
    --from-literal=OPENAI_API_KEY=<key> \\
    --from-literal=OPENAI_API_BASE=<base-url>
EOF
  exit 1
}

ensure_namespace
ensure_scc
ensure_pvc
ensure_pull_secret
check_eval_secrets
log "bootstrap complete for namespace $NAMESPACE"
