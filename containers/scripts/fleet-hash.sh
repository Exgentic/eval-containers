#!/usr/bin/env bash
# fleet-hash — deterministic build-input hashes for every fleet image
# (delivery/RULES.md rules 11–14).
#
# hash(target) = sha256 of the sorted git tree hashes of the target's build
# context and every transitive in-repo base context — a pure function of the
# committed tree at REF, read off the bake graph (principle 15.d keeps each
# target's `contexts` aligned with its Dockerfile's FROMs). A flat set is
# sensitivity-equivalent to a Merkle chain here: wiring changes edit bake
# files, which live inside a hashed context. External FROMs are emitted as
# unresolved refs; digest resolution needs the network and happens at release
# time (rule 11), keeping this script offline.
#
# Usage:
#   fleet-hash.sh                          # every static bake target
#   fleet-hash.sh combo <bench> <agent>    # eval + eval-standalone rows
#   fleet-hash.sh per-task <bench> <task>  # one per-task image row
#
# Output (TSV): target  hash  context-hash  bases-hash  externals
# Env: REF (default HEAD), REPO_ROOT (default: the repo containing this script)
set -euo pipefail
shopt -s nullglob

REF="${REF:-HEAD}"
REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$REPO_ROOT"

die() { echo "fleet-hash: $*" >&2; exit 2; }
sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
hash_of() { sha < "$1" | cut -d' ' -f1; }
row() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }
M=$(mktemp -d) && trap 'rm -rf "$M"' EXIT && mkdir "$M/full" "$M/bases"

# ── graph: one awk over every per-artifact bake file → target|context|deps ──
# (one target per file, principle 15.a; the parameterized combination file
# sits directly in containers/core/, outside the subdir glob)
FILES=(containers/core/*/docker-bake.hcl containers/gateways/*/docker-bake.hcl
  containers/agents/*/docker-bake.hcl containers/benchmarks/*/docker-bake.hcl
  containers/models/*/docker-bake.hcl)
[ "${#FILES[@]}" -gt 0 ] || die "no bake files under $REPO_ROOT/containers"
awk '
  FNR==1 { tgt="" }
  /^target "/ && tgt=="" {
    split($0, q, "\""); tgt=q[2]
    if (tgt in seen) { print "fleet-hash: duplicate target " tgt > "/dev/stderr"; exit 2 }
    seen[tgt]=1; ctx[tgt]=""; deps[tgt]=""
  }
  $1=="context" && $2=="=" && ctx[tgt]=="" { split($0, q, "\""); ctx[tgt]=q[2] }
  {
    s=$0
    while (match(s, /"target:[A-Za-z0-9_-]+"/)) {
      d=substr(s, RSTART+8, RLENGTH-9)
      if (index(" " deps[tgt] " ", " " d " ")==0) deps[tgt]=deps[tgt] d " "
      s=substr(s, RSTART+RLENGTH)
    }
  }
  END { for (t in ctx) print t "|" ctx[t] "|" deps[t] }
' "${FILES[@]}" | LC_ALL=C sort > "$M/graph"

# ── tree hashes: one git call over every context, paired by row order ───────
# shellcheck disable=SC2046  # context paths never contain spaces
git rev-parse $(awk -F'|' -v r="$REF" '{print r ":" $2}' "$M/graph") > "$M/hashes" 2>/dev/null || {
  while IFS='|' read -r t ctx _; do
    git rev-parse "$REF:$ctx" >/dev/null 2>&1 || die "context $ctx of $t is not in $REF (uncommitted?)"
  done < "$M/graph"
  die "git rev-parse failed"
}
paste -d'|' <(cut -d'|' -f1 "$M/graph") "$M/hashes" > "$M/trees"

# ── closures: recursive walk in awk → one sorted tree-hash file per target ──
# full/<t> holds the target's own context tree + every transitive base tree;
# bases/<t> holds the base trees only (the cascade component).
while IFS='|' read -r t _; do : > "$M/full/$t"; : > "$M/bases/$t"; done < "$M/graph"
awk -F'|' '
  FNR==NR { ctx[$1]=$2; deps[$1]=$3; order[++n]=$1; next }
  { tree[$1]=$2 }
  END { for (i=1; i<=n; i++) { t=order[i]; delete hit; walk(t, t, 1) } }
  function walk(root, t, isroot,  m, p, j) {
    if (t in hit) return; hit[t]=1
    if (!(t in ctx)) { print "fleet-hash: " root " depends on unknown target " t > "/dev/stderr"; exit 2 }
    print "F|" root "|" tree[t]
    if (!isroot) print "B|" root "|" tree[t]
    m = split(deps[t], p, " ")
    for (j=1; j<=m; j++) if (p[j]!="") walk(root, p[j], 0)
  }
' "$M/graph" "$M/trees" | LC_ALL=C sort -u \
  | awk -F'|' -v m="$M" '{ f = m "/" ($1=="F" ? "full" : "bases") "/" $2; print $3 >> f }'

# ── externals: one awk over every context Dockerfile → dir|image ────────────
# A FROM on a continuation line (trailing backslash above) is inside a RUN —
# e.g. SQL `FROM 'hf://…'` in duckdb heredocs — never a Dockerfile instruction.
DFS=()
while IFS='|' read -r t ctx _; do
  [ -f "$ctx/Dockerfile" ] || die "$ctx/Dockerfile missing (target $t)"
  DFS+=("$ctx/Dockerfile")
done < "$M/graph"
awk '
  FNR==1 { delete alias; cont=0 }
  !cont && toupper($1)=="FROM" {
    img=$2; if (img ~ /^--platform/) img=$3
    for (i=1; i<=NF; i++) if (toupper($i)=="AS") alias[$(i+1)]=1
    if (!(img in alias) && img!="scratch" && index(img,"${REGISTRY}")==0) {
      d=FILENAME; sub(/\/Dockerfile$/, "", d); print d "|" img
    }
  }
  { cont = ($0 ~ /\\[[:space:]]*$/) }
' "${DFS[@]}" | LC_ALL=C sort -u > "$M/ext"

# ── one sha pass over every closure file, then a single join → the TSV ──────
(cd "$M" && sha full/* bases/*) > "$M/sums"
awk '
  BEGIN { FS="|" }
  FILENAME ~ /graph$/ { ctxdir[$1]=$2; order[++n]=$1; next }
  FILENAME ~ /trees$/ { tree[$1]=$2; next }
  FILENAME ~ /ext$/   { ext[$1] = ($1 in ext) ? ext[$1] "," $2 : $2; next }
  {
    split($0, a, / +/)
    if (split(a[2], b, "/") == 2) { if (b[1]=="full") full[b[2]]=a[1]; else bases[b[2]]=a[1] }
  }
  END {
    for (i=1; i<=n; i++) {
      t=order[i]; e=ext[ctxdir[t]]
      print t "\t" full[t] "\t" tree[t] "\t" bases[t] "\t" (e=="" ? "-" : e)
    }
  }
' "$M/graph" "$M/trees" "$M/ext" "$M/sums" > "$M/all.tsv"

col() { awk -F'\t' -v t="$1" -v c="$2" '$1==t { print $c }' "$M/all.tsv"; }
target_for_dir() {
  awk -F'|' -v d="$1" '$2==d { print $1; found=1 } END { exit !found }' "$M/graph" \
    || die "no bake target with context $1"
}
blobs() { git rev-parse "$@" 2>/dev/null || die "blob not in $REF"; }

case "${1:-all}" in
all)
  cat "$M/all.tsv"
  ;;
combo)
  [ $# -eq 3 ] || die "usage: fleet-hash.sh combo <benchmark> <agent>"
  b=$(target_for_dir "containers/benchmarks/$2")
  a=$(target_for_dir "containers/agents/$3")
  # The combination context is all of containers/core (over-broad); the real
  # inputs are the combination files' blobs plus the parents' closures. The
  # parent list mirrors combination.docker-bake.hcl's variable defaults —
  # changing that list edits a hashed blob, so every combo hash moves with it.
  blobs "$REF:containers/core/combination.Dockerfile" \
    "$REF:containers/core/combination.docker-bake.hcl" | LC_ALL=C sort > "$M/eval.ctx"
  LC_ALL=C sort -u "$M/full/$b" "$M/full/$a" "$M/full/gosu" > "$M/eval.bases"
  LC_ALL=C sort -u "$M/eval.ctx" "$M/eval.bases" > "$M/eval.full"
  row "evals/$2--$3" "$(hash_of "$M/eval.full")" "$(hash_of "$M/eval.ctx")" \
    "$(hash_of "$M/eval.bases")" "-"
  blobs "$REF:containers/core/standalone.Dockerfile" > "$M/sa.ctx"
  LC_ALL=C sort -u "$M/eval.full" "$M/full/otel" "$M/full/process-compose" \
    "$M/full/model-bifrost" > "$M/sa.bases"
  LC_ALL=C sort -u "$M/sa.ctx" "$M/sa.bases" > "$M/sa.full"
  row "evals/$2--$3-standalone" "$(hash_of "$M/sa.full")" "$(hash_of "$M/sa.ctx")" \
    "$(hash_of "$M/sa.bases")" "-"
  ;;
per-task)
  [ $# -eq 3 ] || die "usage: fleet-hash.sh per-task <benchmark> <task-id>"
  [ -z "${SKILLS_BENCH_REF:-}" ] || die "SKILLS_BENCH_REF is set — an out-of-tree ref override defeats input hashing; pin the ref in the benchmark dir"
  t=$(target_for_dir "containers/benchmarks/$2")
  h=$(col "$t" 2)
  row "per-task/$2/$3" "$(printf '%s %s' "$h" "$3" | sha | cut -d' ' -f1)" \
    "$(col "$t" 3)" "$(col "$t" 4)" "$(col "$t" 5)"
  ;;
*)
  die "unknown command $1 (expected: all | combo | per-task)"
  ;;
esac
