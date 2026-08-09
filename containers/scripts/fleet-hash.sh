#!/usr/bin/env bash
# fleet-hash — deterministic build-input hashes for every fleet image
# (delivery/RULES.md rules 11–14).
#
# hash(target) = sha256 of the sorted git tree hashes of the target's build
# context and every transitive in-repo base context — a pure function of the
# committed tree at REF (the containers/ tree is materialized from REF via
# `git archive`, so worktree state is invisible), read off the bake graph
# (principle 15.d keeps each target's `contexts` aligned with its Dockerfile's
# FROMs). A flat set is sensitivity-equivalent to a Merkle chain here: wiring
# changes edit bake files, which live inside a hashed context. External FROMs
# are emitted with same-Dockerfile ARG defaults expanded; refs that still
# carry `${…}` are per-build by design. Digest resolution needs the network
# and happens at release time (rule 11), keeping this script offline.
#
# Usage:
#   fleet-hash.sh                          # every static bake target
#   fleet-hash.sh combo <bench> <agent> [task]  # eval + eval-standalone rows
#                                          # (task ⇒ the per-task combo variant)
#   fleet-hash.sh per-task <bench> <task>  # one per-task image row
#   fleet-hash.sh graph                    # target|context|deps — the context
#                                          # column is also the registry ref
#                                          # path (minus containers/); gated
#                                          # against `bake --print` by
#                                          # tests/static/fleet-hash.sweep.sh
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
M=$(mktemp -d) && trap 'rm -rf "$M"' EXIT && mkdir "$M/full" "$M/bases" "$M/src"

# ── materialize containers/ at REF: all parsing below reads this tree ───────
git archive "$REF" containers 2>/dev/null | tar -x -C "$M/src" \
  || die "cannot read containers/ at $REF"
S="$M/src"

# ── graph: one awk over every per-artifact bake file → target|context|deps ──
# (one target per file, principle 15.a; the parameterized combination file
# sits directly in containers/core/, outside the subdir glob)
FILES=("$S"/containers/core/*/docker-bake.hcl "$S"/containers/gateways/*/docker-bake.hcl
  "$S"/containers/agents/*/docker-bake.hcl "$S"/containers/benchmarks/*/docker-bake.hcl
  "$S"/containers/models/*/docker-bake.hcl)
[ "${#FILES[@]}" -gt 0 ] || die "no bake files under containers/ at $REF"
awk '
  FNR==1 { tgt="" }
  /^target "/ {
    if (tgt != "") { print "fleet-hash: " FILENAME " declares a second target — one per file (principle 15.a)" > "/dev/stderr"; exit 2 }
    split($0, q, "\""); tgt=q[2]
    if (tgt in seen) { print "fleet-hash: duplicate target " tgt > "/dev/stderr"; exit 2 }
    seen[tgt]=1; ctx[tgt]=""; deps[tgt]=""
  }
  tgt != "" && $1=="context" && $2=="=" && ctx[tgt]=="" { split($0, q, "\""); ctx[tgt]=q[2] }
  tgt != "" {
    s=$0; sub(/#.*/, "", s)
    while (match(s, /"target:[^"]+"/)) {
      d=substr(s, RSTART+8, RLENGTH-9)
      if (index(" " deps[tgt] " ", " " d " ")==0) deps[tgt]=deps[tgt] d " "
      s=substr(s, RSTART+RLENGTH)
    }
  }
  END {
    for (t in ctx) {
      if (ctx[t]=="") { print "fleet-hash: target " t " has no context line" > "/dev/stderr"; exit 2 }
      print t "|" ctx[t] "|" deps[t]
    }
  }
' "${FILES[@]}" | LC_ALL=C sort > "$M/graph"

# ── tree hashes: one git call over every context, paired by row order ───────
PATHS=()
while IFS='|' read -r t ctx _; do PATHS+=("$REF:$ctx"); done < "$M/graph"
git rev-parse "${PATHS[@]}" > "$M/hashes" 2>/dev/null || {
  while IFS='|' read -r t ctx _; do
    git rev-parse "$REF:$ctx" >/dev/null 2>&1 || die "context $ctx of $t is not in $REF"
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
  | awk -F'|' -v m="$M" '{
      f = m "/" ($1=="F" ? "full" : "bases") "/" $2
      if (f != prev) { if (prev != "") close(prev); prev = f }
      print $3 >> f
    }'

# ── externals: one awk over every context Dockerfile → dir|image ────────────
# Only an unindented uppercase FROM outside a backslash continuation is an
# instruction — SQL `FROM` fragments and Python `from … import` in heredoc
# RUN bodies are neither. `${VAR}` is expanded from same-file ARG defaults;
# a ref still carrying `${…}` is per-build by design (e.g. per-task bases).
DFS=()
while IFS='|' read -r t ctx _; do
  [ -f "$S/$ctx/Dockerfile" ] || die "$ctx/Dockerfile missing at $REF (target $t)"
  DFS+=("$S/$ctx/Dockerfile")
done < "$M/graph"
awk -v strip="$S/" '
  FNR==1 { delete alias; delete arg; cont=0 }
  /^ARG [A-Za-z_]+=/ { eq=index($2,"="); arg[substr($2,1,eq-1)]=substr($2,eq+1) }
  !cont && /^FROM[ \t]/ {
    img=$2; if (img ~ /^--platform/) img=$3
    if (index(img, "${REGISTRY}") == 0) {
      while (match(img, /\$\{[A-Za-z_]+\}/)) {
        v=substr(img, RSTART+2, RLENGTH-3)
        if (!(v in arg)) break
        img = substr(img, 1, RSTART-1) arg[v] substr(img, RSTART+RLENGTH)
      }
      if (!(img in alias) && img != "scratch") {
        d=substr(FILENAME, length(strip)+1); sub(/\/Dockerfile$/, "", d)
        print d "|" img
      }
    }
    for (i=1; i<=NF; i++) if ($i=="AS") alias[$(i+1)]=1
  }
  { cont = ($0 ~ /\\[ \t]*$/) }
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
    if (split(a[2], b, "/") != 2) { print "fleet-hash: unparsable sums line: " $0 > "/dev/stderr"; exit 2 }
    if (b[1]=="full") full[b[2]]=a[1]
    else if (b[1]=="bases") bases[b[2]]=a[1]
    else { print "fleet-hash: unparsable sums line: " $0 > "/dev/stderr"; exit 2 }
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
  local t
  t=$(awk -F'|' -v d="$1" '$2==d { print $1 }' "$M/graph")
  [ -n "$t" ] || die "no bake target with context $1"
  [ "$(printf '%s\n' "$t" | wc -l)" -eq 1 ] || die "multiple targets with context $1"
  printf '%s' "$t"
}
blobs() { git rev-parse "$@" 2>/dev/null || die "blob not in $REF"; }
# Combo parents come from combination.docker-bake.hcl's *_IMAGE defaults, so a
# changed default re-points the closure at the new target automatically.
parent_target() {
  local p
  # shellcheck disable=SC2016  # the ${REGISTRY}/${TAG} literals are the match
  p=$(grep "\"$1\"" "$S/containers/core/combination.docker-bake.hcl" \
    | sed -n 's|.*"${REGISTRY}/\(.*\):${TAG}".*|\1|p')
  [ -n "$p" ] || die "cannot derive $1 from combination.docker-bake.hcl"
  target_for_dir "containers/$p"
}

case "${1:-all}" in
all)
  cat "$M/all.tsv"
  ;;
graph)
  cat "$M/graph"
  ;;
combo)
  { [ $# -ge 3 ] && [ $# -le 4 ] && [ -n "$2" ] && [ -n "$3" ]; } \
    || die "usage: fleet-hash.sh combo <benchmark> <agent> [task]"
  task="${4:-}"
  case "$task" in *[[:space:]]*) die "task id must not contain whitespace" ;; esac
  b=$(target_for_dir "containers/benchmarks/$2")
  a=$(target_for_dir "containers/agents/$3")
  gosu=$(parent_target GOSU_IMAGE)
  # The combination Dockerfiles COPY from runner/ and entrypoint/ inside the
  # containers/core context, so those trees are combo inputs alongside the
  # Dockerfile + bake-file blobs and the parents' closures.
  blobs "$REF:containers/core/combination.Dockerfile" \
    "$REF:containers/core/combination.docker-bake.hcl" \
    "$REF:containers/core/runner" "$REF:containers/core/entrypoint" \
    | LC_ALL=C sort > "$M/eval.ctx"
  LC_ALL=C sort -u "$M/full/$b" "$M/full/$a" "$M/full/$gosu" > "$M/eval.bases"
  LC_ALL=C sort -u "$M/eval.ctx" "$M/eval.bases" > "$M/eval.full"
  # A per-task combo mixes the task id into the hash the same way per-task
  # does, and names its rows with the release's <bench>-<tid> convention.
  eb="$2"; eval_hash=$(hash_of "$M/eval.full")
  if [ -n "$task" ]; then
    eb="$2-$(printf '%s' "$task" | tr '[:upper:]' '[:lower:]')"
    eval_hash=$(printf '%s %s' "$eval_hash" "$task" | sha | cut -d' ' -f1)
  fi
  row "evals/$eb--$3" "$eval_hash" "$(hash_of "$M/eval.ctx")" \
    "$(hash_of "$M/eval.bases")" "-"
  blobs "$REF:containers/core/standalone.Dockerfile" > "$M/sa.ctx"
  LC_ALL=C sort -u "$M/eval.full" "$M/full/$(parent_target OTEL_IMAGE)" \
    "$M/full/$(parent_target PROCESS_COMPOSE_IMAGE)" \
    "$M/full/$(parent_target MODEL_IMAGE)" > "$M/sa.bases"
  LC_ALL=C sort -u "$M/sa.ctx" "$M/sa.bases" > "$M/sa.full"
  sa_hash=$(hash_of "$M/sa.full")
  [ -z "$task" ] || sa_hash=$(printf '%s %s' "$sa_hash" "$task" | sha | cut -d' ' -f1)
  row "evals/$eb--$3-standalone" "$sa_hash" "$(hash_of "$M/sa.ctx")" \
    "$(hash_of "$M/sa.bases")" "-"
  ;;
per-task)
  { [ $# -eq 3 ] && [ -n "$2" ] && [ -n "$3" ]; } || die "usage: fleet-hash.sh per-task <benchmark> <task-id>"
  case "$3" in *[[:space:]]*) die "task id must not contain whitespace" ;; esac
  [ -z "${SKILLS_BENCH_REF:-}" ] || die "SKILLS_BENCH_REF is set — an out-of-tree ref override defeats input hashing; pin the ref in the benchmark dir"
  t=$(target_for_dir "containers/benchmarks/$2")
  h=$(col "$t" 2)
  row "per-task/$2/$3" "$(printf '%s %s' "$h" "$3" | sha | cut -d' ' -f1)" \
    "$(col "$t" 3)" "$(col "$t" 4)" "$(col "$t" 5)"
  ;;
*)
  die "unknown command $1 (expected: all | combo | per-task | graph)"
  ;;
esac
