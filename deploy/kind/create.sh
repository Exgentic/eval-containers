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
# gateway, whose start script appends it to the system roots. The CA lives only
# on the cluster — never baked into an image. See the README section above.
#
#   OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-endpoint ./deploy/kind/create.sh
#   ./deploy/kind/create.sh --cluster eval --namespace my-ns
#   EVAL_UPSTREAM_CA=./corp-ca.pem OPENAI_API_KEY=... OPENAI_API_BASE=... ./deploy/kind/create.sh
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
                    run.sh mounts it into the gateway, whose start script appends
                    it to the system roots. Never baked into an image. See the
                    README § 'Private-CA upstreams'.

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

# GET {base}/models from a throwaway pod — through the same egress AND trust store
# the gateway pod will have: when the eval-upstream-ca ConfigMap exists it is
# mounted at /etc/eval-ca and appended to curl's CA bundle, the same way the
# gateway's start script appends it to the system roots. Otherwise the probe would
# report a bogus cert failure on a cluster whose CA is already installed. Emits the
# pod's stdout: the response body then a trailer `__probe__=<curl-rc> <http-status>`
# — curl's exit code says whether TLS + network reached the server; the HTTP status
# then separates a good response from an auth rejection (401/403) or a server error.
#
# A hand-applied Pod (not `kubectl run`) because run's --overrides fights the
# --command/-i merge; a full manifest gives unambiguous control of the CA volume.
probe_models_from_cluster() {
  local name=eval-upstream-probe
  # When the CA is installed, mount it and point curl at it (CURL_CA_BUNDLE). For a
  # private-CA endpoint that one CA is what verifies it; a public endpoint has no
  # ConfigMap installed, so this only narrows trust to the CA the gateway uses too.
  # Optional YAML fragments are built as whole lines (no braces inside ${:+}, which
  # would swallow a closing brace). The command is static + env-driven.
  # When the CA is installed, trust public roots + the private CA — matching the
  # gateway (whose start script builds the same combined bundle), not a bare
  # CURL_CA_BUNDLE that would REPLACE public roots. The image runs as non-root and
  # its system bundle is read-only, so build a COMBINED bundle in writable /tmp and
  # point curl there.
  # KEY and URL come from the eval-secrets Secret (created in step 2, before this
  # probe) via secretKeyRef — the SAME path the gateway reads them from — never
  # interpolated as plaintext into the pod spec. That keeps the key out of
  # `kubectl get pod -o yaml`/audit logs, and means neither the key nor the URL is
  # spliced into the manifest text, so a value containing a quote or YAML
  # metacharacter can't corrupt the manifest. curl strips a trailing slash off the
  # base with ${URL%/} at request time (the Secret stores it verbatim).
  local vol="" mnt="" cmd
  cmd='curl -sS --max-time 15 -w "\n__probe__=%{exitcode} %{http_code}" -H "Authorization: Bearer $KEY" "${URL%/}/models"'
  if kube get configmap eval-upstream-ca >/dev/null 2>&1; then
    vol='  volumes: [{ name: upstream-ca, configMap: { name: eval-upstream-ca } }]'
    mnt='      volumeMounts: [{ name: upstream-ca, mountPath: /etc/eval-ca, readOnly: true }]'
    cmd='cat /etc/ssl/certs/ca-certificates.crt /etc/eval-ca/ca.pem > /tmp/ca-bundle.pem; export CURL_CA_BUNDLE=/tmp/ca-bundle.pem; '"$cmd"
  fi
  kube delete pod "$name" --ignore-not-found >/dev/null 2>&1
  kube apply -f - >/dev/null 2>&1 <<YAML
apiVersion: v1
kind: Pod
metadata: { name: $name }
spec:
  restartPolicy: Never
$vol
  containers:
    - name: probe
      image: $PROBE_IMAGE
      command: ["sh", "-c", "$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')"]
      env:
        - { name: KEY, valueFrom: { secretKeyRef: { name: eval-secrets, key: OPENAI_API_KEY } } }
        - { name: URL, valueFrom: { secretKeyRef: { name: eval-secrets, key: OPENAI_API_BASE } } }
$mnt
YAML
  # Wait for it to finish (bounded), then hand back its logs. Any failure to
  # schedule/pull surfaces as empty logs → the caller reads that as unreachable.
  kube wait --for=jsonpath='{.status.phase}'=Succeeded pod/"$name" --timeout=40s >/dev/null 2>&1 \
    || kube wait --for=jsonpath='{.status.phase}'=Failed pod/"$name" --timeout=5s >/dev/null 2>&1 || true
  kube logs "$name" 2>/dev/null
  kube delete pod "$name" --ignore-not-found >/dev/null 2>&1
}

# Extract OpenAI-style model ids from a /models response body (one per line).
models_from_body() {
  printf '%s' "$1" | grep -oE '"id"[[:space:]]*:[[:space:]]*"[^"]+"' \
    | sed -E 's/.*"([^"]+)"$/\1/' || true
}

# Log a model summary: the count, then up to 10 ids. Arg: newline-separated ids.
log_models() {  # arg: <ids>
  local ids="$1" count
  count=$(printf '%s' "$ids" | grep -c . || true)
  log "the upstream offers $count model(s)$([[ "$count" -gt 10 ]] && echo ' (first 10)'):"
  printf '%s' "$ids" | head -10 | while IFS= read -r m; do [[ -n "$m" ]] && log "  - $m"; done
}

# Verify OPENAI_API_BASE from the HOST first — the host is the source of truth. If
# the host itself can't reach a usable /models, nothing downstream can, so we stop
# there and tell the user to fix host access. Only once the host works do we probe
# the cluster to see whether it, too, reaches the endpoint (CA/egress). Read-only.
# Returns 0 only when an eval could actually run; non-zero on any failure.
probe_and_report() {
  $DRY_RUN && { log "[dry-run] skip upstream reachability/model probe"; return 0; }

  # ── Host first ──────────────────────────────────────────────────────────────
  log "checking $OPENAI_API_BASE from the host (GET /models) …"
  local hbody="" hrc=0 host_http
  hbody="$(curl -sS -w $'\n__http__=%{http_code}' --max-time 15 \
             -H "Authorization: Bearer $OPENAI_API_KEY" "${OPENAI_API_BASE%/}/models" 2>/dev/null)" || hrc=$?
  host_http="${hbody##*__http__=}"; host_http="${host_http%%[!0-9]*}"; [[ "$host_http" =~ ^[0-9]+$ ]] || host_http=000
  hbody="${hbody%$'\n'__http__=*}"

  if [[ "$hrc" -ne 0 || "$host_http" == 000 ]]; then
    log "NOT reachable from the host (curl $hrc, http $host_http). Model list: unavailable."
    log "Fix host access to OPENAI_API_BASE first — check the URL, your network/VPN, and"
    log "that the endpoint is up — then re-run. (Nothing else was attempted.)"
    return 1
  fi
  case "$host_http" in
    2*) : ;;  # reachable + OK — fall through to model list + cluster check
    401|403)
      log "reachable from the host but the key was rejected (HTTP $host_http). Fix OPENAI_API_KEY,"
      log "then re-run. (The cluster was not probed.)"; return 1;;
    5*)
      log "reachable from the host but it returned HTTP $host_http — the endpoint is up but"
      log "unhealthy (model server down/scaling?). Retry later. (The cluster was not probed.)"; return 1;;
    *)
      log "reachable from the host but GET /models returned HTTP $host_http — check OPENAI_API_BASE"
      log "points at the API root (…/v1), not a web page. (The cluster was not probed.)"; return 1;;
  esac

  local models; models="$(models_from_body "$hbody")"
  local mcount; mcount=$(printf '%s' "$models" | grep -c . || true)
  if [[ "$mcount" -eq 0 ]]; then
    # A valid-but-empty /models list is specifically {"object": "list", "data": []}.
    # Match that shape, not a bare "data"/"object"/"[]" anywhere — an error payload
    # like {"object": "error", …} carries "object" too and would otherwise be
    # misreported as "works but serves nothing" instead of "not a /models payload".
    if printf '%s' "$hbody" | tr -d '[:space:]' | grep -q '"object":"list"'; then
      log "reachable from the host (HTTP $host_http); /models is valid but lists no models."
      log "The endpoint works but serves nothing to evaluate right now. (Cluster not probed.)"
    else
      log "reachable from the host (HTTP $host_http) but the response is not an OpenAI /models"
      log "payload — check OPENAI_API_BASE points at the API root (…/v1). (Cluster not probed.)"
    fi
    return 1
  fi
  log "reachable from the host (HTTP $host_http)."
  log_models "$models"

  # ── Then the cluster ─────────────────────────────────────────────────────────
  # The host works. Does the cluster reach it too (CA installed / egress OK)?
  # Retry a few times on a bare connect failure (curl rc=0, http=000): right after
  # --recreate the node's CNI/DNS/egress are still settling, so the first probe on
  # a seconds-old cluster often opens a connection that never completes. A real
  # CA/auth/network fault reproduces on every attempt, so retrying costs nothing
  # but absorbs the cold-cluster warmup window.
  log "checking the same endpoint from the cluster …"
  local curl_rc cluster_http attempt
  for attempt in 1 2 3; do
    # probe_models_from_cluster's own exit status is its final `kube delete pod`,
    # NOT curl's — so read the outcome only from the __probe__=<rc> <http> trailer
    # the probe pod prints. No trailer means the pod never ran curl (failed to
    # schedule/pull, or empty logs); that is an inconclusive "no response", so
    # default to rc=0/http=000 and let the retry/again-warming-up path handle it,
    # rather than mistaking kube's exit code for a curl network error.
    local resp
    resp="$(probe_models_from_cluster || true)"
    curl_rc=0 cluster_http=000
    if [[ "$resp" == *"__probe__="* ]]; then
      local trailer="${resp##*__probe__=}"
      read -r curl_rc cluster_http <<<"$trailer"
      [[ "$curl_rc" =~ ^[0-9]+$ ]] || curl_rc=1
      [[ "$cluster_http"   =~ ^[0-9]+$ ]] || cluster_http=000
    fi
    # Only a bare connect failure is worth retrying; any other outcome is terminal.
    [[ "$curl_rc" -eq 0 && "$cluster_http" == 000 && "$attempt" -lt 3 ]] || break
    log "no response yet (cluster may still be warming up) — retry $attempt/2 …"
    sleep 5
  done

  if [[ "$curl_rc" -eq 0 && "$cluster_http" != 000 ]]; then
    log "OK — the cluster reaches it too (HTTP $cluster_http). Ready to run evals."
    return 0
  fi

  # Cluster can't reach what the host can. Classify by curl's exit code; http=000
  # with a 0 exit still means "no HTTP response", so treat it as a connect failure
  # (never print 'curl 0' as the reason — it reads as a contradiction).
  case "$curl_rc" in
    35|51|58|59|60|66|77|80|82|83|91)
      log "but the cluster CANNOT (curl $curl_rc: TLS/cert). The cluster is missing a private CA"
      log "the host trusts — install it: deploy/kind/README.md § 'Private-CA upstreams', then re-run.";;
    0)
      log "but the cluster got no response (HTTP 000) — it opened a connection but the TLS/HTTP"
      log "exchange did not complete. Often a private-CA/egress issue: see deploy/kind/README.md"
      log "§ 'Private-CA upstreams'. Re-run to retry.";;
    *)
      log "but the cluster CANNOT (curl $curl_rc) — a cluster egress/network issue. If it is a"
      log "private-CA endpoint, see deploy/kind/README.md § 'Private-CA upstreams'.";;
  esac
  return 1
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
# ConfigMap that run.sh mounts into the gateway at /etc/eval-ca/ca.pem for a
# private-CA upstream; the gateway's start script appends it to the system roots
# (a combined bundle), so the file need hold only the extra CA(s). Default-off:
# unset → no ConfigMap and the gateway trusts only public roots, as before. See
# deploy/kind/README.md § 'Private-CA upstreams'.
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
# README. Writes nothing to the cluster. The cluster + Secret are already created
# (side effects kept) — but if the upstream isn't usable an eval can't run, so we
# withhold the "ready" line and exit non-zero. Capture the status (don't let set
# -e abort) so the caller reports the right ending.
probe_status=0
probe_and_report || probe_status=$?

if [[ "$probe_status" -ne 0 ]]; then
  log "cluster provisioned, but the upstream above is not usable yet — fix it, then"
  log "re-run this script (idempotent) before submitting an eval."
  exit "$probe_status"
fi

log "ready. submit an eval with: ./deploy/kind/run.sh --benchmark <b> --agent <a> --model <m> --task 0 --watch"
