#!/usr/bin/env bash
# tests/static/security/trivy.sh — trivy security gate for the fleet (issue #114).
#
# Net-new coverage. Nothing in the suite checks IaC misconfiguration or image
# CVEs today; gitleaks (verify step 22) scans secret *values* only. trivy is the
# standard tool for "is this Dockerfile/compose misconfigured" and "does this
# BUILT image ship known-vulnerable packages", so we run it directly rather than
# reimplementing either check in Rust. Two lanes, mirroring
# structure.release.sweep.sh:
#
#   (default) config  — CONTRIBUTION lane. `trivy config` over containers/
#                       (every Dockerfile + compose.yaml). Static, daemon-free,
#                       fast; runs on every PR. Misconfig scanner, which also
#                       carries the DS-0031 "secret in build-arg/env" IaC check
#                       (gitleaks can't see an empty-valued secret-named ARG).
#                       Fails on HIGH,CRITICAL.
#
#   image             — RELEASE lane. `trivy image` CVE scan of whatever fleet
#                       images are present locally (a smoke check; release CI
#                       builds the fleet first, so it scans the lot there). Slow,
#                       needs the vuln DB + built images — opt-in, never on a PR.
#
# Usage:
#   tests/static/security/trivy.sh            # config lane (default)
#   tests/static/security/trivy.sh config     # same, explicit
#   tests/static/security/trivy.sh image      # image-CVE lane (release; built images)
#
# Severity gate is HIGH,CRITICAL for both lanes; override with EVAL_TRIVY_SEVERITY.
# Accepted/by-design misconfig findings live in .github/.trivyignore (each
# documented there). Fail loud: no `|| true`, no `2>/dev/null` swallowing
# (.agents/verification/RULES.md:57).
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd) || exit 2
SEVERITY=${EVAL_TRIVY_SEVERITY:-HIGH,CRITICAL}
REG=${EVAL_REGISTRY:-ghcr.io/exgentic}

command -v trivy >/dev/null ||
  { echo "trivy not found — required for the security gate (https://trivy.dev/latest/getting-started/installation/)"; exit 1; }

# --skip-version-check silences the "new trivy released" notice so the gate's
# output is deterministic; it does NOT skip the vuln-DB refresh (image lane).
COMMON=(--severity "$SEVERITY" --exit-code 1 --skip-version-check
        --ignorefile "$ROOT/.github/.trivyignore")

# config lane: misconfig (+ DS-0031 secret-arg) over every Dockerfile + compose
# file. Scoped to containers/ — that is the fleet (the issue's "Dockerfiles and
# compose files"); deploy/*.yaml is Kubernetes policy, kubeconform's lane, not
# this one. No daemon, no network beyond the one-time rego-check pull.
config_lane() {
  echo "trivy config: scanning containers/ (Dockerfiles + compose) at severity $SEVERITY"
  trivy config "$ROOT/containers" "${COMMON[@]}"
  local rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "trivy config: FAIL — HIGH/CRITICAL misconfiguration(s) above. Fix the"
    echo "  Dockerfile/compose, or — only if accepted/by-design — add a documented"
    echo "  AVD-* entry to $ROOT/.github/.trivyignore."
    exit 1
  fi
  echo "trivy config: clean (no HIGH/CRITICAL outside .trivyignore)"
}

# image lane: CVE scan of fleet images that happen to be built locally. Skips
# any image that isn't present (a contributor rarely has all ~150); release CI
# builds the fleet first, so nothing is skipped there. "Did it build" is the
# build sweep's job; this owns "the built image is free of HIGH/CRITICAL CVEs".
image_lane() {
  # trivy talks to the daemon to read local images; honor the docker CLI's
  # active context when DOCKER_HOST isn't set (podman/colima/Docker Desktop put
  # the socket elsewhere) — same shim as structure.release.sweep.sh.
  if [ -z "${DOCKER_HOST:-}" ]; then
    DOCKER_HOST=$(docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null) && export DOCKER_HOST
  fi

  local checked=0 skipped=0 fail=0
  scan_one() {
    local img=$1
    docker image inspect "$img" >/dev/null 2>&1 || { skipped=$((skipped + 1)); return; }
    checked=$((checked + 1))
    echo "── trivy image $img ──"
    # --pkg-types os,library covers both distro packages and language deps.
    if ! trivy image "$img" "${COMMON[@]}" --pkg-types os,library --scanners vuln; then
      fail=$((fail + 1))
      echo "FAIL $img — HIGH/CRITICAL CVE(s) above"
    fi
  }

  # Enumerate the same fleet the release build pushes (core, agents, benchmarks,
  # models, gateways) at :latest; scan whichever are present locally.
  for kind in core agents benchmarks models gateways; do
    for d in "$ROOT"/containers/"$kind"/*/; do
      [ -d "$d" ] || continue
      name=$(basename "$d"); case $name in _*|.*) continue ;; esac
      scan_one "$REG/$kind/$name:latest"
    done
  done

  echo "trivy image: $checked checked, $skipped skipped (not built locally), $fail failed"
  [ "$fail" -eq 0 ] ||
    { echo "trivy image: FAIL — fix the base/package pins (a CVE bump is a patch release, RULES.md principle 9) or add a documented .trivyignore entry"; exit 1; }
  [ "$checked" -gt 0 ] ||
    { echo "trivy image: no fleet images present locally to scan — build the fleet first (release lane)"; exit 1; }
}

case "${1:-config}" in
  config) config_lane ;;
  image)  image_lane ;;
  *) echo "usage: $0 [config|image]" >&2; exit 2 ;;
esac
