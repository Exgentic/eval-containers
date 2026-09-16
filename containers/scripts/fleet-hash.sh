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
# are emitted with same-Dockerfile ARG defaults expanded and folded into the
# hash through containers/externals.tsv (ref<TAB>digest, written by
# external-drift.sh), so an upstream base bump is a changed input (rule 11)
# while this script stays offline; refs that still carry `${…}` are per-build
# by design and have no digest to fold.
#
# Usage:
#   fleet-hash.sh                          # every static bake target
#   fleet-hash.sh combo <bench> <agent> [task]  # eval + eval-standalone rows
#                                          # (task ⇒ the per-task combo variant)
#   fleet-hash.sh per-task <bench> <task>… # per-task image rows (ids may
#                                          # also arrive on stdin, one per line)
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
# The lockfile (ref<TAB>digest, written by external-drift.sh) pins what each
# external resolves to; only a real digest is an input — `local` marks an
# image built beside the fleet, whose inputs the tree already covers.
: > "$M/extd"
[ ! -f "$S/containers/externals.tsv" ] || awk '
  FILENAME ~ /externals\.tsv$/ { split($0, l, "\t"); if (l[2] ~ /^sha256:/) lock[l[1]]=l[2]; next }
  { split($0, a, "|"); if (a[2] in lock) print a[1] "|" lock[a[2]] }
' "$S/containers/externals.tsv" "$M/ext" > "$M/extd"

# ── closures: recursive walk in awk → one sorted input file per target ──────
# full/<t> holds the target's own context tree + every transitive base tree,
# plus the pinned digest of every external those contexts build FROM;
# bases/<t> holds the base half only (the cascade component).
while IFS='|' read -r t _; do : > "$M/full/$t"; : > "$M/bases/$t"; done < "$M/graph"
awk -F'|' '
  FILENAME ~ /graph$/ { ctx[$1]=$2; deps[$1]=$3; order[++n]=$1; next }
  FILENAME ~ /trees$/ { tree[$1]=$2; next }
  { xd[$1] = xd[$1] " " $2 }
  END { for (i=1; i<=n; i++) { t=order[i]; delete hit; walk(t, t, 1) } }
  function emit(root, isroot, v) { print "F|" root "|" v; if (!isroot) print "B|" root "|" v }
  function walk(root, t, isroot,  m, p, j, x) {
    if (t in hit) return; hit[t]=1
    if (!(t in ctx)) { print "fleet-hash: " root " depends on unknown target " t > "/dev/stderr"; exit 2 }
    emit(root, isroot, tree[t])
    m = split(xd[ctx[t]], x, " ")
    for (j=1; j<=m; j++) if (x[j]!="") emit(root, isroot, x[j])
    m = split(deps[t], p, " ")
    for (j=1; j<=m; j++) if (p[j]!="") walk(root, p[j], 0)
  }
' "$M/graph" "$M/trees" "$M/extd" | LC_ALL=C sort -u \
  | awk -F'|' -v m="$M" '{
      f = m "/" ($1=="F" ? "full" : "bases") "/" $2
      if (f != prev) { if (prev != "") close(prev); prev = f }
      print $3 >> f
    }'

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
blobs() { git rev-parse "$@" 2>/dev/null || die "blob not in $REF: $*"; }
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
  # One call, many combos: with no arguments the triples arrive on stdin, one
  # `<benchmark> <agent> [task]` per line. bash resolves the closures once (the
  # setup above) and one python process derives every pair and every task from
  # them — so a whole release's worth of combos costs one materialisation of
  # the tree, not one process per combo, which is where 3 s of every combo
  # build went. The arithmetic is byte-for-byte what the shell pipeline did:
  # LC_ALL=C sort -u unions, sha256 of the resulting file, and for a task
  # sha256("<pair hash> <task>") — the static tests hold both forms to it.
  { [ $# -eq 1 ] || { [ $# -ge 3 ] && [ $# -le 4 ] && [ -n "$2" ] && [ -n "$3" ]; }; } \
    || die "usage: fleet-hash.sh combo <benchmark> <agent> [task]  (or triples on stdin)"
  if [ $# -ge 3 ]; then printf '%s %s %s\n' "$2" "$3" "${4:-}" > "$M/triples"; else cat > "$M/triples"; fi
  # The combination Dockerfiles COPY from runner/ and entrypoint/ inside the
  # containers/core context, so those trees are combo inputs alongside the
  # Dockerfile + bake-file blobs and the parents' closures.
  blobs "$REF:containers/core/combination.Dockerfile" \
    "$REF:containers/core/combination.docker-bake.hcl" \
    "$REF:containers/core/runner" "$REF:containers/core/entrypoint" \
    | LC_ALL=C sort > "$M/eval.ctx"
  blobs "$REF:containers/core/standalone.Dockerfile" > "$M/sa.ctx"
  # Combo parents come from combination.docker-bake.hcl's *_IMAGE defaults.
  GOSU_T=$(parent_target GOSU_IMAGE) EDGE_T=$(parent_target EDGE_IMAGE) \
  OTEL_T=$(parent_target OTEL_IMAGE) PC_T=$(parent_target PROCESS_COMPOSE_IMAGE) \
  MODEL_T=$(parent_target MODEL_IMAGE) M="$M" python3 - <<'PY' || exit 2
import hashlib, os, sys
M = os.environ["M"]
graph = {}                                   # context dir -> target (bake graph)
for line in open(f"{M}/graph"):
    t, ctx, _ = line.rstrip("\n").split("|", 2)
    graph.setdefault(ctx, []).append(t)
def target_for_dir(d):
    ts = graph.get(d, [])
    if not ts: sys.exit(f"fleet-hash: no bake target with context {d}")
    if len(ts) > 1: sys.exit(f"fleet-hash: multiple targets with context {d}")
    return ts[0]
full_cache = {}
def full(t):                                 # lines of $M/full/<target>
    if t not in full_cache:
        full_cache[t] = [l.rstrip("\n") for l in open(f"{M}/full/{t}") if l != "\n"]
    return full_cache[t]
def sort_u(*lists):                          # LC_ALL=C sort -u
    return sorted(set(x for l in lists for x in l), key=lambda x: x.encode())
def sha_lines(lines):                        # sha256sum < file written by sort -u
    return hashlib.sha256("".join(x + "\n" for x in lines).encode()).hexdigest()
def sha_file(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()
def sha_str(s):                              # printf '%s' ... | sha256sum
    return hashlib.sha256(s.encode()).hexdigest()
lower = str.maketrans("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmnopqrstuvwxyz")  # tr '[:upper:]' '[:lower:]'
E = os.environ
eval_ctx = [l.rstrip("\n") for l in open(f"{M}/eval.ctx") if l != "\n"]
ctxh, sactxh = sha_file(f"{M}/eval.ctx"), sha_file(f"{M}/sa.ctx")
parents = [full(E["GOSU_T"]), full(E["EDGE_T"])]
sa_parents = [full(E["OTEL_T"]), full(E["PC_T"]), full(E["MODEL_T"])]
pairs = {}
for line in open(f"{M}/triples"):
    parts = line.split()
    if not parts: continue
    b, a, task = parts[0], parts[1], (parts[2] if len(parts) > 2 else "")
    if len(parts) > 3: sys.exit("fleet-hash: task id must not contain whitespace")
    if (b, a) not in pairs:
        bt, at = target_for_dir(f"containers/benchmarks/{b}"), target_for_dir(f"containers/agents/{a}")
        eval_bases = sort_u(full(bt), full(at), *parents)
        eval_full = sort_u(eval_ctx, eval_bases)
        sa_bases = sort_u(eval_full, *sa_parents)
        sa_full = sort_u([l.rstrip("\n") for l in open(f"{M}/sa.ctx") if l != "\n"], sa_bases)
        pairs[(b, a)] = (sha_lines(eval_full), sha_lines(eval_bases), sha_lines(sa_full), sha_lines(sa_bases))
    ef, ebh, sf, sbh = pairs[(b, a)]
    eb = b
    if task:
        eb = f"{b}-{task.translate(lower)}"
        ef, sf = sha_str(f"{ef} {task}"), sha_str(f"{sf} {task}")
    print(f"evals/{eb}--{a}\t{ef}\t{ctxh}\t{ebh}\t-")
    print(f"evals/{eb}--{a}-standalone\t{sf}\t{sactxh}\t{sbh}\t-")
PY
  ;;
per-task)
  { [ $# -ge 2 ] && [ -n "$2" ]; } || die "usage: fleet-hash.sh per-task <benchmark> <task-id>… (or ids on stdin)"
  [ -z "${SKILLS_BENCH_REF:-}" ] || die "SKILLS_BENCH_REF is set — an out-of-tree ref override defeats input hashing; pin the ref in the benchmark dir"
  t=$(target_for_dir "containers/benchmarks/$2")
  # Every task of a benchmark shares that benchmark's closure and differs only
  # by the id mixed in, so a whole task list costs one pass, not one per task.
  h=$(col "$t" 2); ctxh=$(col "$t" 3); basesh=$(col "$t" 4); exts=$(col "$t" 5)
  b="$2"; shift 2
  emit() {
    # An explicitly empty argument is misuse; a blank line in a piped list is
    # noise and is skipped below.
    [ -n "$1" ] || die "usage: fleet-hash.sh per-task <benchmark> <task-id>… (or ids on stdin)"
    case "$1" in *[[:space:]]*) die "task id must not contain whitespace" ;; esac
    row "per-task/$b/$1" "$(printf '%s %s' "$h" "$1" | sha | cut -d' ' -f1)" \
      "$ctxh" "$basesh" "$exts"
  }
  if [ $# -gt 0 ]; then
    for tk in "$@"; do emit "$tk"; done
  else
    while read -r tk; do [ -n "$tk" ] || continue; emit "$tk"; done
  fi
  ;;
*)
  die "unknown command $1 (expected: all | combo | per-task | graph)"
  ;;
esac
