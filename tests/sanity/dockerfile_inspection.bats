#!/usr/bin/env bats
# Framework-free port of tests/sanity/dockerfile_inspection.rs (issue #114) —
# the 22-rule Dockerfile lint sweep over containers/{benchmarks,agents,models}/*/
# expressed as bats instead of a Rust integration test. Each Rust `#[test]`
# becomes one bats `@test`, preserving the rule↔test pairing AND the Red
# (fail the run) vs Yellow (advisory, print-but-pass) partition of
# `inspect_every_dockerfile`.
#
# Engine is plain shell (grep/find/awk over file text); bats only provides the
# test reporting/isolation. Adds only — deletes nothing, changes no rule. The
# .rs port stays alongside until teardown (handled separately).
#
# Two intentional deviations from the .rs, both deliberate:
#   * The `hardcoded_secret` rule is dropped — gitleaks (in
#     .pre-commit-config.yaml) owns secret scanning, so its rule and its
#     `rule_hardcoded_secret_fires` unit test are NOT ported.
#   * Two new unit @tests cover the shared `strip_heredocs` and
#     `join_run_continuations` helpers (the hard part of the port).
#
# Fail-loud (verification RULES rule 8 — "test code MUST NOT swallow errors"):
# Dockerfiles are slurped via `read_df`, which fails the run if a file cannot be
# read. No blanket `2>/dev/null` / `|| true` masks a real error anywhere here.
#
# Run: bats tests/sanity/dockerfile_inspection.bats

REPO="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"

# ── file slurp (fail loud) ──────────────────────────────────────────────────
# Read a Dockerfile into stdout. Unreadable file => non-zero exit + message on
# stderr; callers must not swallow it. (check.bats's has_line uses 2>/dev/null;
# this does better per RULES rule 8.)
read_df() {
  local f=$1
  [ -f "$f" ] || { echo "read_df: not a file: $f" >&2; return 1; }
  cat -- "$f"
}

# ── heredoc stripping ───────────────────────────────────────────────────────
# Port of Rust `strip_heredocs`: Dockerfiles write install scripts via
# `cat > file <<'NAME'` heredocs; the body is file content, not a RUN command,
# so install-rule checks MUST skip it. Replace every heredoc body line with a
# blank line (preserving line count), keeping the opener and the closing tag.
# Tag = first run of [A-Za-z0-9_] after `<<`, skipping a leading `-` and an
# optional leading `'` or `"`; closing line is the body line whose trim == tag.
strip_heredocs() {
  awk '
    BEGIN { intag = "" }
    {
      if (intag != "") {
        t = $0; sub(/^[ \t]+/, "", t); sub(/[ \t]+$/, "", t)
        if (t == intag) intag = ""
        print ""                      # blank out the body / closing line
        next
      }
      i = index($0, "<<")
      if (i > 0) {
        after = substr($0, i + 2)
        sub(/^-/, "", after)          # `<<-TAG`
        sub(/^['\''"]/, "", after)    # `<<'\''TAG'\''` or `<<"TAG"`
        tag = ""
        n = length(after)
        for (j = 1; j <= n; j++) {
          c = substr(after, j, 1)
          if (c ~ /[A-Za-z0-9_]/) tag = tag c; else break
        }
        if (tag != "") { intag = tag; print; next }
      }
      print
    }
  '
}

# ── multiline RUN join ──────────────────────────────────────────────────────
# Port of Rust `.replace("\\\r\n"," ").replace("\\\n"," ")`: a line ending in a
# backslash continues onto the next physical line, so join the two with a single
# space. This makes `RUN apt-get install -y \ <newline> && rm -rf …` one logical
# line so the same-RUN cleanup/flag checks see the whole command.
join_run_continuations() {
  awk '
    {
      line = line $0
      if (line ~ /\\$/) { sub(/\\$/, " ", line); next }
      print line; line = ""
    }
    END { if (line != "") print line }
  '
}

# strip heredocs, then join continuations (the order the install rules use)
strip_and_join() { strip_heredocs | join_run_continuations; }

# Per-file cache of strip_and_join output, set once by the sweep so the four
# stripping predicates do not each re-fork awk (one strip per file, not four).
# Unset => the predicate computes its own, keeping it self-contained for the
# unit tests that call it directly.
__SJ=""
_sj() { if [ -n "$__SJ" ]; then printf '%s' "$__SJ"; else printf '%s\n' "$1" | strip_and_join; fi; }

# trim leading+trailing whitespace from $1 into the global T. Assigning to a
# global (not `x=$(trim …)`) matters: command substitution forks a subshell on
# every call, and these run per-line over 131 files — a fork-per-line would make
# the sweep minutes slow under bash 3.2. Pure parameter expansion, zero forks.
T=""
trim() { local s=$1; s=${s#"${s%%[![:space:]]*}"}; s=${s%"${s##*[![:space:]]}"}; T=$s; }

# ── rule predicates (one per Rust fn; echo "fire" + return 0 when it fires) ──
# Each takes the raw Dockerfile text on stdin (or via $1 = text, $2 = dir where
# the dir name matters). They print nothing and return 1 when the rule is clean.

# missing_dock_type: no `LABEL eval.type=`
r_missing_dock_type() {
  grep -qF 'LABEL eval.type=' <<<"$1" && return 1 || return 0
}

# untagged_from: a FROM whose image ref (after --flags) has no :tag and no @digest
r_untagged_from() {
  local line rest image tail tok
  while IFS= read -r line; do
    line=${line#"${line%%[![:space:]]*}"}            # ltrim only (Rust trim_start)
    case $line in "FROM "*) ;; *) continue ;; esac
    trim "${line#FROM }"; rest=$T
    [ -z "$rest" ] && continue
    case $rest in scratch*|'$'*) continue ;; esac
    image=""
    for tok in $rest; do case $tok in --*) ;; *) image=$tok; break ;; esac; done
    tail=${image##*/}                                 # after last slash
    case $tail in *:*|*@*) ;; *) return 0 ;; esac
  done <<<"$1"
  return 1
}

# legacy_env_var: whole-identifier $TASK_ID / $BENCHMARK (and ${...}) not
# prefixed by EVAL_ and not extended by an identifier char. Raw scan (no strip).
r_legacy_env_var() {
  local t=$1 needle rest before after nextc continue_outer extended
  for needle in '$TASK_ID' '${TASK_ID' '$BENCHMARK' '${BENCHMARK'; do
    rest=$t
    while :; do
      case $rest in
        *"$needle"*) before=${rest%%"$needle"*} ;;
        *) break ;;
      esac
      after=${rest#*"$needle"}
      # prefixed by EVAL_ ?
      case $before in *EVAL_) continue_outer=1 ;; *) continue_outer=0 ;; esac
      # followed by identifier continuation char ?
      nextc=${after:0:1}
      case $nextc in [A-Za-z0-9_]) extended=1 ;; *) extended=0 ;; esac
      if [ "$continue_outer" -eq 0 ] && [ "$extended" -eq 0 ]; then return 0; fi
      # advance past this occurrence (Rust does rest = &rest[i+1..])
      rest=${rest#*"$needle"}
      rest="${needle:1}$rest"   # keep all but the first char of the needle, like &rest[i+1..]
    done
  done
  return 1
}

# label_dir_mismatch: eval.benchmark.name / eval.agent.name present and != dir.
# (Rust: absent label => match=true => no finding.)
r_label_dir_mismatch() {
  local t=$1 dir=$2 line l key rest val
  while IFS= read -r line; do
    trim "$line"; l=$T
    case $l in "LABEL "*) ;; *) continue ;; esac
    for key in 'eval.benchmark.name=' 'eval.agent.name='; do
      case $l in
        *"$key"*)
          rest=${l#*"$key"}
          # trim leading quotes/space, then take up to next quote/space
          rest=${rest#"${rest%%[!\"\'[:space:]]*}"}
          val=${rest%%[\"\'[:space:]]*}
          [ "$val" = "$dir" ] && return 1 || return 0
          ;;
      esac
    done
  done <<<"$t"
  return 1
}

# apt_no_cleanup: apt-get install on a (stripped+joined) line without
# `rm -rf /var/lib/apt/lists` on that same logical line.
r_apt_no_cleanup() {
  local joined line
  joined=$(_sj "$1")
  while IFS= read -r line; do
    trim "$line"; line=$T
    case $line in '#'*) continue ;; esac
    case $line in *"apt-get install"*) ;; *) continue ;; esac
    case $line in *"rm -rf /var/lib/apt/lists"*) ;; *) return 0 ;; esac
  done <<<"$joined"
  return 1
}

# pip_no_cache_flag: pip/pip3 install (stripped+joined) without --no-cache-dir
# or --no-cache (uv's spelling). Skips `pip uninstall` lines.
r_pip_no_cache_flag() {
  local joined line
  joined=$(_sj "$1")
  while IFS= read -r line; do
    trim "$line"; line=$T
    case $line in '#'*) continue ;; esac
    case $line in *"pip install"*|*"pip3 install"*) ;; *) continue ;; esac
    case $line in *"pip uninstall"*) continue ;; esac
    case $line in *--no-cache-dir*|*--no-cache*) ;; *) return 0 ;; esac
  done <<<"$joined"
  return 1
}

# unpinned_pip: a pip/pip3 install token that is a bare package name (no
# ==/>=/~= pin), excluding -r requirements, git+…@rev, .whl/.tgz/.tar.gz, and
# transient build tools that are pip-uninstalled later in the file. Raw scan.
r_unpinned_pip() {
  local t=$1 line after tok
  while IFS= read -r line; do
    trim "$line"; line=$T
    case $line in '#'*) continue ;; esac
    case $line in *"pip install"*|*"pip3 install"*) ;; *) continue ;; esac
    case $line in *" -r "*) continue ;; esac
    case $line in *"pip uninstall"*) continue ;; esac
    case $line in
      *"pip install"*) after=${line#*"pip install"} ;;
      *) after=${line#*"pip3 install"} ;;
    esac
    for tok in $after; do
      case $tok in -*|/*|'$'*) continue ;; esac
      case $tok in '\'|'&&'|'||'|';') break ;; esac
      case $tok in *uninstall) break ;; esac
      case $tok in *'&&'*) break ;; esac
      case $tok in *'=='*|*'>='*|*'~='*) continue ;; esac
      case $tok in *git+*) case $tok in *@*|*'#'*) continue ;; esac ;; esac
      case $tok in *.tgz|*.whl|*.tar.gz) continue ;; esac
      # bare package name: [a-z0-9_-], len>1
      case $tok in
        *[!a-z0-9_-]*) ;;                  # has a non-name char => not a bare pkg
        ?*?*)                              # len > 1
          if _pip_uninstalled_later "$t" "$tok"; then continue; fi
          return 0
          ;;
      esac
    done
  done <<<"$t"
  return 1
}
# helper: is $2 pip-uninstalled somewhere in text $1 (transient build tool)?
_pip_uninstalled_later() {
  local t=$1 pkg=$2 line l
  while IFS= read -r line; do
    trim "$line"; l=$T
    case $l in *"pip uninstall"*|*"pip3 uninstall"*) ;; *) continue ;; esac
    case $l in *"$pkg"*) return 0 ;; esac
  done <<<"$t"
  return 1
}

# unpinned_npm: `npm install -g` / `npm i -g` token that is an unpinned package
# (no `@version` after the package, allowing a leading scope `@scope/`).
r_unpinned_npm() {
  local t=$1 line after tok stripped
  while IFS= read -r line; do
    trim "$line"; line=$T
    case $line in '#'*) continue ;; esac
    case $line in *"npm install -g"*|*"npm i -g"*) ;; *) continue ;; esac
    after=${line#*-g}
    for tok in $after; do
      case $tok in -*|/*|'$'*) continue ;; esac
      case $tok in '\'|'&&'|'||'|';') break ;; esac
      case $tok in *.tgz|*.tar.gz) continue ;; esac
      stripped=${tok#@}                    # drop a leading scope @
      case $stripped in *@*) continue ;; esac
      case $tok in ?*?*) return 0 ;; esac  # len > 1
    done
  done <<<"$t"
  return 1
}

# todo_or_fixme: a `#` comment with a standalone TODO/FIXME/XXX token (FUTURE:
# block exempt).
r_todo_or_fixme() {
  local line trimmed tok words w
  while IFS= read -r line; do
    trimmed=${line#"${line%%[![:space:]]*}"}
    case $trimmed in '#'*) ;; *) continue ;; esac
    case $trimmed in *FUTURE:*) continue ;; esac
    # split on non-alphanumerics; standalone-token match
    words=$(printf '%s' "$trimmed" | tr -c 'A-Za-z0-9' ' ')
    for w in $words; do
      case $w in TODO|FIXME|XXX) return 0 ;; esac
    done
  done <<<"$1"
  return 1
}

# todo_string_literal: a non-comment line containing the quoted literal "TODO"
# or 'TODO' (placeholder task data written into the image).
r_todo_string_literal() {
  local line trimmed
  while IFS= read -r line; do
    trim "$line"; trimmed=$T
    case $trimmed in '#'*) continue ;; esac
    case $trimmed in *'"TODO"'*|*"'TODO'"*) return 0 ;; esac
  done <<<"$1"
  return 1
}

# silent_pip_fallback: a pip/pip3 install line that also swallows errors via
# 2>/dev/null or `|| true`. Case-insensitive (Rust lowercases the line; we use
# nocasematch on the original — every needle is already lowercase, so the result
# is identical, and we avoid a `tr` fork per line).
r_silent_pip_fallback() {
  local line; shopt -s nocasematch
  while IFS= read -r line; do
    case $line in *"pip install"*|*"pip3 install"*) ;; *) continue ;; esac
    case $line in *"2>/dev/null"*) shopt -u nocasematch; return 0 ;; esac
    case $line in *"|| true"*) shopt -u nocasematch; return 0 ;; esac
  done <<<"$1"
  shopt -u nocasematch; return 1
}

# install_order_pip_before_apt: a top-level `RUN … pip install` appears before a
# top-level `RUN … apt-get install` (volatile pip layer ahead of stable apt).
r_install_order_pip_before_apt() {
  local line t i=0 pip_first=-1; shopt -s nocasematch
  while IFS= read -r line; do
    i=$((i + 1))
    trim "$line"; t=$T
    case $t in '#'*) continue ;; esac
    case $t in 'run '*) ;; *) continue ;; esac
    case $t in
      *"pip install"*|*"pip3 install"*)
        [ "$pip_first" -lt 0 ] && pip_first=$i
        ;;
    esac
    case $t in
      *"apt-get install"*)
        if [ "$pip_first" -ge 0 ] && [ "$pip_first" -lt "$i" ]; then
          shopt -u nocasematch; return 0
        fi
        ;;
    esac
  done <<<"$1"
  shopt -u nocasematch; return 1
}

# phantom_pip_uninstall: a top-level `RUN … pip uninstall` with no `pip install`
# on the same RUN line (reclaims no space — must combine with the install).
r_phantom_pip_uninstall() {
  local line t; shopt -s nocasematch
  while IFS= read -r line; do
    trim "$line"; t=$T
    case $t in '#'*) continue ;; esac
    case $t in 'run '*) ;; *) continue ;; esac
    case $t in *"pip uninstall"*) ;; *) continue ;; esac
    case $t in *"pip install"*) ;; *) shopt -u nocasematch; return 0 ;; esac
  done <<<"$1"
  shopt -u nocasematch; return 1
}

# missing_data_revision_when_fetching_mutable_ref: a (stripped+joined) RUN that
# fetches a mutable ref (refs/convert/parquet, ?revision=main|master, github raw
# /main/|/master/) without a pinned (non-mutable) eval.benchmark.data_revision.
r_missing_data_revision_when_fetching_mutable_ref() {
  local t=$1 joined line has_mutable=0 rest val
  joined=$(_sj "$t")
  while IFS= read -r line; do
    trim "$line"; line=$T
    case $line in '#'*) continue ;; esac
    # Rust lowercases ONLY for the `run ` gate; the URL substring checks stay
    # case-sensitive. Match RUN with an explicit case glob (no nocasematch).
    case $line in [Rr][Uu][Nn]\ *) ;; *) continue ;; esac
    case $line in
      *refs/convert/parquet*|*'?revision=main'*|*'?revision=master'*)
        has_mutable=1; break ;;
    esac
    case $line in
      *raw.githubusercontent.com/*)
        case $line in *"/main/"*|*"/master/"*) has_mutable=1; break ;; esac
        ;;
    esac
  done <<<"$joined"
  [ "$has_mutable" -eq 1 ] || return 1
  # allow when a non-mutable data_revision label is present
  while IFS= read -r line; do
    case $line in
      *'eval.benchmark.data_revision='*)
        rest=${line#*=}
        trim "$rest"; val=$T; val=${val#\"}; val=${val%\"}; val=${val#\'}; val=${val%\'}
        trim "$val"; val=$T
        case $val in
          ''|latest|main|master|HEAD) ;;
          *) return 1 ;;
        esac
        ;;
    esac
  done <<<"$t"
  return 0
}

# stale_data_revision (Yellow): eval.benchmark.data_revision is empty/latest/
# main/master/HEAD. (First such label decides, matching the Rust early return.)
r_stale_data_revision() {
  local line rest val
  while IFS= read -r line; do
    case $line in
      *'eval.benchmark.data_revision='*)
        rest=${line#*eval.benchmark.data_revision=}
        rest=${rest#"${rest%%[!\"\'[:space:]]*}"}      # trim leading quote/space
        val=${rest%%[\"\'[:space:]]*}                  # up to next quote/space
        case $val in
          ''|latest|main|master|HEAD) return 0 ;;
          *) return 1 ;;
        esac
        ;;
    esac
  done <<<"$1"
  return 1
}

# python_full_base (Yellow): `FROM python:X` without -slim/-alpine/-dev.
r_python_full_base() {
  local line l image
  while IFS= read -r line; do
    l=${line#"${line%%[![:space:]]*}"}
    case $l in "FROM "*) ;; *) continue ;; esac
    trim "${l#FROM }"; image=${T%% *}
    case $image in
      python:*)
        case $image in *-slim*|*-alpine*|*-dev*) ;; *) return 0 ;; esac
        ;;
    esac
  done <<<"$1"
  return 1
}

# upstream_base_unpinned (Yellow): eval.benchmark.upstream_base ends in :latest
# or carries neither a : tag nor an @ digest.
r_upstream_base_unpinned() {
  local line rest val
  while IFS= read -r line; do
    case $line in
      *'eval.benchmark.upstream_base='*)
        rest=${line#*eval.benchmark.upstream_base=}
        rest=${rest#"${rest%%[!\"\'[:space:]]*}"}      # trim leading quote/space
        val=${rest%%[\"\']*}                           # value up to next quote
        case $val in
          *:latest) return 0 ;;
          *:*|*@*) return 1 ;;
          *) return 0 ;;
        esac
        ;;
    esac
  done <<<"$1"
  return 1
}

# ── type predicates + version-axis rules (RULES.md principle 9) ──────────────
_is_agent() { case $1 in *'LABEL eval.type="agent"'*) return 0 ;; *) return 1 ;; esac; }
_is_model() { case $1 in *'LABEL eval.type="model"'*) return 0 ;; *) return 1 ;; esac; }
_is_replay_model() { [ "$2" = "replay" ]; }
_is_gateway_flavor_model() { case $1 in *'LABEL gateway.kind='*) return 0 ;; *) return 1 ;; esac; }

# agent_missing_version_arg: agent image without `ARG AGENT_VERSION`.
r_agent_missing_version_arg() {
  _is_agent "$1" || return 1
  case $1 in *'ARG AGENT_VERSION'*) return 1 ;; *) return 0 ;; esac
}
# model_missing_litellm_version_label: non-replay, non-gateway model image
# without LABEL eval.model.litellm_version.
r_model_missing_litellm_version_label() {
  _is_model "$1" || return 1
  _is_replay_model "$1" "$2" && return 1
  _is_gateway_flavor_model "$1" && return 1
  case $1 in *'LABEL eval.model.litellm_version='*) return 1 ;; *) return 0 ;; esac
}
# model_missing_litellm_version_default: same scope, without
# ENV EVAL_LITELLM_VERSION_DEFAULT.
r_model_missing_litellm_version_default() {
  _is_model "$1" || return 1
  _is_replay_model "$1" "$2" && return 1
  _is_gateway_flavor_model "$1" && return 1
  case $1 in *'ENV EVAL_LITELLM_VERSION_DEFAULT='*) return 1 ;; *) return 0 ;; esac
}

# ── rule catalog: id | severity | predicate  (mirrors the Rust RULES array) ──
# hardcoded_secret is intentionally absent — gitleaks owns secret scanning.
RED_RULES=(
  "missing_dock_type|r_missing_dock_type"
  "untagged_from|r_untagged_from"
  "legacy_env_var|r_legacy_env_var"
  "label_dir_mismatch|r_label_dir_mismatch"
  "apt_no_cleanup|r_apt_no_cleanup"
  "pip_no_cache_flag|r_pip_no_cache_flag"
  "unpinned_pip|r_unpinned_pip"
  "unpinned_npm|r_unpinned_npm"
  "todo_or_fixme|r_todo_or_fixme"
  "todo_string_literal|r_todo_string_literal"
  "silent_pip_fallback|r_silent_pip_fallback"
  "agent_missing_version_arg|r_agent_missing_version_arg"
  "model_missing_litellm_version_label|r_model_missing_litellm_version_label"
  "model_missing_litellm_version_default|r_model_missing_litellm_version_default"
)
YELLOW_RULES=(
  "stale_data_revision|r_stale_data_revision"
  "python_full_base|r_python_full_base"
  "upstream_base_unpinned|r_upstream_base_unpinned"
  "install_order_pip_before_apt|r_install_order_pip_before_apt"
  "phantom_pip_uninstall|r_phantom_pip_uninstall"
  "missing_data_revision_when_fetching_mutable_ref|r_missing_data_revision_when_fetching_mutable_ref"
)

# ── discovery: every Dockerfile under containers/{benchmarks,agents,models} ──
# Emits "<dir>\t<path>" lines, sorted by path (mirrors walk_dockerfiles()).
discover_dockerfiles() {
  local root d name df
  for root in benchmarks agents models; do
    [ -d "$REPO/containers/$root" ] || continue
    for d in "$REPO/containers/$root"/*/; do
      [ -d "$d" ] || continue
      d=${d%/}                              # strip trailing slash from the glob
      name=${d##*/}
      df="$d/Dockerfile"
      [ -f "$df" ] && printf '%s\t%s\n' "$name" "$df"
    done
  done | sort -t$'\t' -k2
}

# Apply a rule set to a Dockerfile; print "<path> (<rule>)" per fire.
# bash 3.2 has no namerefs, so the rule entries are passed as trailing args.
#   apply_rules <text> <dir> <path> <entry...>
# Fail loud (RULES rule 8): an entry naming a missing predicate aborts the run
# rather than silently matching nothing.
apply_rules() {
  local text=$1 dir=$2 path=$3 entry id fn
  shift 3
  for entry in "$@"; do
    id=${entry%%|*}; fn=${entry#*|}
    if ! declare -f "$fn" >/dev/null; then
      echo "apply_rules: no such predicate function: $fn (rule $id)" >&2
      return 2
    fi
    if "$fn" "$text" "$dir"; then printf '%s (%s)\n' "$path" "$id"; fi
  done
}

# ── fleet sweep engine (port of inspect_every_dockerfile) ────────────────────
# Red findings fail the run; Yellow findings print but pass (advisory) — the
# exact partition of the Rust sweep. This runs as a plain function, NOT inside a
# bats @test: bats wraps every command in a DEBUG trap for line tracking, which
# turns the ~131-file × 20-rule loop into a multi-minute crawl. The @test shells
# out to script mode (below) once, so bats traces a single command and the loop
# runs untraced — ~2s instead of minutes.
run_fleet_sweep() {
  local dir path text count=0 red="" yellow="" out rc nred nyellow
  while IFS=$'\t' read -r dir path; do
    [ -n "$path" ] || continue
    count=$((count + 1))
    # Fail loud (RULES rule 8): a Dockerfile we discovered but cannot read,
    # or a predicate that errors, aborts the run — never a silent skip.
    if ! text=$(read_df "$path"); then
      echo "read error: $path"; return 1
    fi
    # Compute the stripped+joined view once per file; the $(...) subshells below
    # inherit __SJ, so the four install/data rules reuse it (one awk pass/file).
    __SJ=$(printf '%s\n' "$text" | strip_and_join)
    out=$(apply_rules "$text" "$dir" "$path" "${RED_RULES[@]}"); rc=$?
    [ "$rc" -eq 0 ] || { echo "$out"; echo "RED rule engine error on $path (rc=$rc)"; return 1; }
    [ -n "$out" ] && red+="$out"$'\n'
    out=$(apply_rules "$text" "$dir" "$path" "${YELLOW_RULES[@]}"); rc=$?
    [ "$rc" -eq 0 ] || { echo "$out"; echo "YELLOW rule engine error on $path (rc=$rc)"; return 1; }
    [ -n "$out" ] && yellow+="$out"$'\n'
  done < <(discover_dockerfiles)

  # strip the trailing blank line left by the per-file accumulation
  red=$(printf '%s' "$red" | grep -v '^$' || true)
  yellow=$(printf '%s' "$yellow" | grep -v '^$' || true)
  nred=$(printf '%s' "$red" | grep -c . || true)
  nyellow=$(printf '%s' "$yellow" | grep -c . || true)

  [ "$count" -gt 0 ] || { echo "no Dockerfiles found under containers/{benchmarks,agents,models}"; return 1; }

  echo "─── dockerfile inspection over $count files ───"
  if [ -n "$yellow" ]; then
    echo "$nyellow yellow findings:"
    printf '%s\n' "$yellow" | sed 's/^/  /'
  fi

  if [ -z "$red" ]; then
    echo "✓ all $count Dockerfiles healthy ($nyellow yellow warnings)"
    return 0
  fi

  echo "$nred red findings:"
  printf '%s\n' "$red" | sed 's/^/  /'
  return 1
}

# ── script mode (must precede the @test definitions) ─────────────────────────
# Executed directly as `bash <file> __sweep`: run the sweep and exit. Placed here
# so a plain-script run hits this guard and exits BEFORE bash parses any `@test`
# line. Under bats the file is sourced with no args (verified: $# == 0 at source
# time), so the guard is inert and only the @test definitions take effect.
if [ "${1:-}" = "__sweep" ]; then
  REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  run_fleet_sweep
  exit $?
fi

# ─────────────────────────────────────────────────────────────────────────────
# Unit tests — one @test per Rust #[test] (minus rule_hardcoded_secret_fires),
# plus the two helper @tests. Each builds a minimal Dockerfile and asserts the
# rule fires / stays silent, exactly like the Rust unit tests.
# ─────────────────────────────────────────────────────────────────────────────

@test "rule missing_dock_type fires" {
  run r_missing_dock_type "$(printf 'FROM alpine:3\nRUN echo hi\n')"
  [ "$status" -eq 0 ]
}

@test "rule untagged_from fires" {
  run r_untagged_from "$(printf 'FROM ubuntu\nLABEL eval.type="agent"\n')"
  [ "$status" -eq 0 ]
}

@test "rule untagged_from allows scratch" {
  run r_untagged_from "$(printf 'FROM scratch\nLABEL eval.type="agent"\n')"
  [ "$status" -ne 0 ]
}

@test "rule legacy_env_var fires" {
  run r_legacy_env_var "$(printf 'FROM alpine:3\nLABEL eval.type="benchmark"\nRUN echo $TASK_ID\n')"
  [ "$status" -eq 0 ]
}

@test "rule legacy_env_var allows EVAL_ prefix" {
  run r_legacy_env_var "$(printf 'FROM alpine:3\nLABEL eval.type="benchmark"\nRUN echo $EVAL_TASK_ID\n')"
  [ "$status" -ne 0 ]
}

@test "rule label_dir_mismatch fires" {
  run r_label_dir_mismatch "$(printf 'FROM alpine:3\nLABEL eval.type="benchmark"\nLABEL eval.benchmark.name="other"\n')" "mybench"
  [ "$status" -eq 0 ]
}

@test "rule apt_no_cleanup fires" {
  run r_apt_no_cleanup "$(printf 'FROM ubuntu:24.04\nLABEL eval.type="agent"\nRUN apt-get update && apt-get install -y curl\n')"
  [ "$status" -eq 0 ]
}

@test "rule apt_no_cleanup allows inline rm" {
  run r_apt_no_cleanup "$(printf 'FROM ubuntu:24.04\nLABEL eval.type="agent"\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\n')"
  [ "$status" -ne 0 ]
}

@test "rule todo_or_fixme fires" {
  run r_todo_or_fixme "$(printf 'FROM alpine:3\nLABEL eval.type="agent"\n# TODO: fix this\n')"
  [ "$status" -eq 0 ]
}

@test "rule todo_or_fixme allows FUTURE block" {
  run r_todo_or_fixme "$(printf 'FROM alpine:3\nLABEL eval.type="agent"\n# FUTURE: consider swapping to alpine\n')"
  [ "$status" -ne 0 ]
}

# ── helper unit tests (the hard part of the port) ──────────────────────────

@test "strip_heredocs blanks heredoc bodies but keeps RUN commands" {
  # A pip install inside a heredoc body must NOT survive (it is file content);
  # a real RUN pip install outside the heredoc must survive.
  local df out
  df=$(printf '%s\n' \
    'RUN cat > /grade.py <<'"'"'PYEOF'"'"'' \
    'pip install evilunpinned' \
    'PYEOF' \
    'RUN pip install --no-cache-dir realpkg==1.0')
  out=$(printf '%s\n' "$df" | strip_heredocs)
  # body line blanked (explicit fail — a bare `! grep` does not fail a bats test)
  if grep -q 'evilunpinned' <<<"$out"; then echo "heredoc body leaked: $out"; false; fi
  # opener + real RUN preserved
  grep -q "cat > /grade.py" <<<"$out"
  grep -q "RUN pip install --no-cache-dir realpkg==1.0" <<<"$out"
  # line count preserved (4 in, 4 out)
  [ "$(wc -l <<<"$df")" -eq "$(wc -l <<<"$out")" ]
}

@test "join_run_continuations joins backslash-continued RUN lines" {
  local df out
  df=$(printf '%s\n' \
    'RUN apt-get update && apt-get install -y \' \
    '  curl \' \
    '  && rm -rf /var/lib/apt/lists/*')
  out=$(printf '%s\n' "$df" | join_run_continuations)
  # the three physical lines collapse to one logical line
  [ "$(wc -l <<<"$out")" -eq 1 ]
  grep -q 'apt-get install -y' <<<"$out"
  grep -q 'rm -rf /var/lib/apt/lists' <<<"$out"
  # and that single line therefore passes the apt-cleanup rule
  run r_apt_no_cleanup "$df"
  [ "$status" -ne 0 ]
}

# ─────────────────────────────────────────────────────────────────────────────
# Fleet sweep — the @test shim. The engine (run_fleet_sweep) and the script-mode
# guard are defined ABOVE the unit @tests so that, when this file is executed as
# a plain script (`bash <file> __sweep`), bash reaches the guard and exits before
# it ever parses an `@test` line (which is a bats keyword, not a shell builtin).
# The @test below simply shells out to that script mode once — see the rationale
# next to run_fleet_sweep's definition.
# ─────────────────────────────────────────────────────────────────────────────

@test "inspect every dockerfile (Red fails, Yellow advisory)" {
  # Shell out once so the heavy loop runs untraced by bats's DEBUG trap.
  run bash "$BATS_TEST_DIRNAME/dockerfile_inspection.bats" __sweep
  echo "$output"
  [ "$status" -eq 0 ]
}
