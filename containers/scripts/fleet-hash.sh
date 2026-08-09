#!/usr/bin/env bash
# fleet-hash — deterministic build-input hashes for every fleet image
# (delivery/RULES.md rules 11–14).
#
# The hash is a pure function of the committed tree at REF:
#   hash(target) = sha256(context tree hash + sorted recursive base hashes)
# read off the bake graph (each target's context dir + its `target:` edges,
# which principle 15.d keeps aligned with the Dockerfile's FROMs). External
# base images are emitted as refs only — resolving their digests needs the
# network, so it happens at release time (rule 11); this script stays offline.
#
# Usage:
#   fleet-hash.sh                          # every static bake target
#   fleet-hash.sh combo <bench> <agent>    # eval + eval-standalone rows
#   fleet-hash.sh per-task <bench> <task>  # one per-task image row
#
# Output (TSV): target  hash  context-hash  bases-hash  externals
# Env: REF (default HEAD), REPO_ROOT (default: the repo containing this script)
#
# Portable to bash 3.2 (macOS, no associative arrays) and sized for the
# static-stage time budget: one awk parses every bake file, one git call
# hashes every context, maps are eval'd shell variables.
#
# shellcheck disable=SC2034,SC2154  # map variables are assigned and read via eval
set -euo pipefail
shopt -s nullglob

REF="${REF:-HEAD}"
REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$REPO_ROOT"

# Captured before `set --` reuses the positional parameters below.
CMD="${1:-all}" NARGS=$# A2="${2:-}" A3="${3:-}"

die() { echo "fleet-hash: $*" >&2; exit 2; }

sha() {
  local out
  if command -v sha256sum >/dev/null 2>&1; then out=$(sha256sum); printf '%s' "${out%% *}"
  else out=$(openssl dgst -sha256); printf '%s' "${out##*= }"; fi
}

# Map key: target names are [A-Za-z0-9_-] (HCL forbids dots), so dash→underscore
# is the only rewrite; a collision between two live names would need a -/_ pair
# and is caught by the duplicate check below. Directory keys additionally carry
# slashes and dots (models/gpt-5.4), so dkey squashes every non-alnum char.
key() { printf '%s' "${1//-/_}"; }
dkey() { printf '%s' "${1//[^a-zA-Z0-9]/_}"; }

# ── parse the bake graph: one awk over every per-artifact bake file ─────────
# (the parameterized combination file sits directly in containers/core/, so
# the subdir glob naturally excludes it; principle 15.a fixes one target per
# file, first `context =` wins, `contexts =` never matches the context line)
FILES=()
for f in containers/core/*/docker-bake.hcl containers/gateways/*/docker-bake.hcl \
         containers/agents/*/docker-bake.hcl containers/benchmarks/*/docker-bake.hcl \
         containers/models/*/docker-bake.hcl; do FILES+=("$f"); done
[ "${#FILES[@]}" -gt 0 ] || die "no bake files under $REPO_ROOT/containers"

PARSED=$(awk '
  FNR==1 { tgt=""; ctx="" }
  /^target "/ && tgt=="" { split($0, q, "\""); tgt=q[2]; deps[tgt]="" }
  $1=="context" && $2=="=" && ctx=="" { split($0, q, "\""); ctx=q[2]; ctxof[tgt]=ctx }
  {
    s=$0
    while (match(s, /"target:[A-Za-z0-9_-]+"/)) {
      d=substr(s, RSTART+8, RLENGTH-9)
      if (index(" " deps[tgt] " ", " " d " ")==0) deps[tgt]=deps[tgt] d " "
      s=substr(s, RSTART+RLENGTH)
    }
  }
  END { for (t in ctxof) print t "|" ctxof[t] "|" deps[t] }
' "${FILES[@]}" | LC_ALL=C sort)

TARGETS=""
PATHS=()
while IFS='|' read -r tgt ctx deps; do
  [ -n "$tgt" ] || continue
  k=$(key "$tgt")
  eval "prev=\${NAME_$k:-}"
  [ -z "$prev" ] || die "duplicate/colliding target $tgt vs $prev"
  eval "NAME_$k=\$tgt CTX_$k=\$ctx DEPS_$k=\$deps"
  TARGETS="$TARGETS $tgt"
  PATHS+=("$REF:$ctx")
done <<EOF
$PARSED
EOF

# ── batch tree hashes: one git call for every context ───────────────────────
TREES=$(git rev-parse "${PATHS[@]}" 2>/dev/null) || {
  for p in "${PATHS[@]}"; do
    git rev-parse "$p" >/dev/null 2>&1 || die "context ${p#"$REF":} is not in $REF (uncommitted?)"
  done
  die "git rev-parse failed"
}
i=0
# shellcheck disable=SC2086  # TARGETS is a space-separated list, split intended
set -- $TARGETS
while IFS= read -r h; do
  i=$((i+1)); eval "TREE_$(key "$1")=\$h"; shift
done <<EOF
$TREES
EOF

# ── externals: one awk over every context Dockerfile ────────────────────────
# FROM refs that are neither a prior build stage, scratch, nor an in-repo
# ${REGISTRY} image; their ref strings are already inside the context tree
# hash — this column only feeds release-time digest resolution.
DFS=()
for t in $TARGETS; do
  eval "ctx=\$CTX_$(key "$t")"
  [ -f "$ctx/Dockerfile" ] || die "$ctx/Dockerfile missing (target $t)"
  DFS+=("$ctx/Dockerfile")
done
# A FROM on a continuation line (trailing backslash above it) is inside a RUN —
# e.g. SQL `FROM 'hf://…'` in duckdb heredocs — never a Dockerfile instruction.
EXTS=$(awk '
  FNR==1 { delete alias; cont=0 }
  !cont && toupper($1)=="FROM" {
    img=$2; if (img ~ /^--platform/) img=$3
    for (i=1; i<=NF; i++) if (toupper($i)=="AS") alias[$(i+1)]=1
    if (!(img in alias) && img!="scratch" && index(img,"${REGISTRY}")==0)
      print FILENAME "|" img
  }
  { cont = ($0 ~ /\\[[:space:]]*$/) }
' "${DFS[@]}" | LC_ALL=C sort -u)
while IFS='|' read -r df img; do
  [ -n "$df" ] || continue
  k=$(dkey "${df%/Dockerfile}")
  eval "cur=\${EXT_$k:-}"
  eval "EXT_$k=\"\${cur:+\$cur,}\$img\""
done <<EOF
$EXTS
EOF
ext_of() { eval "ctx=\$CTX_$(key "$1")"; eval "printf '%s' \"\${EXT_$(dkey "$ctx"):--}\""; }

# ── recursive hash with memoization ─────────────────────────────────────────
resolve() {
  local t=$1 k dep deps dh="" ctxh bh h
  k=$(key "$t")
  eval "[ -z \"\${HASH_$k:-}\" ]" || return 0
  eval "[ -z \"\${VIS_$k:-}\" ]" || die "dependency cycle at $t"
  eval "VIS_$k=1"
  eval "ctxh=\${TREE_$k:-}"
  [ -n "$ctxh" ] || die "unknown target $t"
  eval "deps=\$DEPS_$k"
  for dep in $deps; do
    resolve "$dep"
    eval "dh=\"\$dh\${HASH_$(key "$dep")} \""
  done
  bh=$(printf '%s' "$dh" | sha)
  h=$(printf '%s %s' "$ctxh" "$dh" | sha)
  eval "CTXH_$k=\$ctxh BASESH_$k=\$bh HASH_$k=\$h VIS_$k="
}

row() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }
out() { local k; k=$(key "$1"); eval "row \"\$1\" \"\$HASH_$k\" \"\$CTXH_$k\" \"\$BASESH_$k\" \"\$(ext_of \"\$1\")\""; }

# Target whose context is the given artifact dir — name-agnostic, so targets
# whose name mangles the dir (dots → underscores) still resolve.
target_for_dir() {
  local dir=$1 t ctx
  for t in $TARGETS; do
    eval "ctx=\$CTX_$(key "$t")"
    [ "$ctx" = "$dir" ] && { printf '%s' "$t"; return; }
  done
  die "no bake target with context $dir"
}

blob() { git rev-parse "$REF:$1" 2>/dev/null || die "$1 is not in $REF"; }

case "$CMD" in
all)
  for t in $TARGETS; do resolve "$t"; done
  for t in $TARGETS; do out "$t"; done
  ;;
combo)
  [ "$NARGS" -eq 3 ] || die "usage: fleet-hash.sh combo <benchmark> <agent>"
  b=$(target_for_dir "containers/benchmarks/$A2")
  a=$(target_for_dir "containers/agents/$A3")
  for t in "$b" "$a" gosu otel process-compose model-bifrost; do resolve "$t"; done
  # The combination context is all of containers/core (over-broad); the real
  # inputs are the two Dockerfiles + the bake file + the parent images.
  eval "bh=\$HASH_$(key "$b")"; eval "ah=\$HASH_$(key "$a")"
  eval_bases=$(printf '%s %s %s' "$bh" "$ah" "$HASH_gosu" | sha)
  eval_ctx=$(printf '%s %s' "$(blob containers/core/combination.Dockerfile)" \
    "$(blob containers/core/combination.docker-bake.hcl)" | sha)
  eval_hash=$(printf '%s %s' "$eval_ctx" "$eval_bases" | sha)
  row "evals/$A2--$A3" "$eval_hash" "$eval_ctx" "$eval_bases" "-"
  sa_bases=$(printf '%s %s %s %s' "$eval_hash" "$HASH_otel" \
    "$HASH_process_compose" "$HASH_model_bifrost" | sha)
  sa_ctx=$(blob containers/core/standalone.Dockerfile)
  row "evals/$A2--$A3-standalone" "$(printf '%s %s' "$sa_ctx" "$sa_bases" | sha)" \
    "$sa_ctx" "$sa_bases" "-"
  ;;
per-task)
  [ "$NARGS" -eq 3 ] || die "usage: fleet-hash.sh per-task <benchmark> <task-id>"
  [ -z "${SKILLS_BENCH_REF:-}" ] || die "SKILLS_BENCH_REF is set — an out-of-tree ref override defeats input hashing; pin the ref in the benchmark dir"
  t=$(target_for_dir "containers/benchmarks/$A2")
  resolve "$t"
  k=$(key "$t")
  eval "th=\$HASH_$k ch=\$CTXH_$k bsh=\$BASESH_$k"
  row "per-task/$A2/$A3" "$(printf '%s %s' "$th" "$A3" | sha)" "$ch" "$bsh" "$(ext_of "$t")"
  ;;
*)
  die "unknown command $CMD (expected: all | combo | per-task)"
  ;;
esac
