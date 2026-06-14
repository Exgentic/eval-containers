# Eval-specific Dockerfile hygiene — conftest/OPA port of the rules in
# tests/sanity/dockerfile_inspection.rs that hadolint does NOT own, for issue #114.
#
# IN SCOPE (the eval-domain rules):
#   deny  (red):
#     - legacy_env_var      $TASK_ID / $BENCHMARK must be $EVAL_*
#     - todo_or_fixme       a # comment with a standalone TODO/FIXME/XXX (FUTURE: exempt)
#     - silent_pip_fallback pip install with 2>/dev/null or || true
#   warn  (yellow):
#     - python_full_base    FROM python:X without -slim/-alpine/-dev
#     - stale_data_revision eval.benchmark.data_revision is ''/latest/main/master/HEAD
#     - missing_data_revision_when_fetching_mutable_ref
#                           a RUN fetches a mutable ref but no data_revision pin
#
# OUT OF SCOPE — left to hadolint (generic image hygiene, per the issue):
#     apt_no_cleanup, pip_no_cache_flag  → "hadolint's job"
#   Also out of scope for THIS migration (not in the issue's list, so not ported
#   here): missing_dock_type, untagged_from, unpinned_pip, unpinned_npm,
#   from_arg_not_global, hardcoded_secret, label_dir_mismatch, todo_string_literal,
#   install_order_pip_before_apt, phantom_pip_uninstall, upstream_base_unpinned,
#   agent_missing_version_arg, model_missing_litellm_version_*.
#
# PARITY NOTE (heredocs): buildkit drops heredoc *bodies* from the parse, so the
# RUN-scanning rules below see only what Rust sees after its strip_heredocs() —
# the two agree on the current tree. The one rule Rust runs over RAW text
# (legacy_env_var) can in principle catch a $TASK_ID buried in a heredoc body that
# conftest cannot see; on this tree both find zero. See the migration report.

package main

import rego.v1

# Severity split: conftest treats `deny` as failures, `warn` as warnings. The Rust
# Red rules map to deny; Yellow rules map to warn.

# ── legacy_env_var (red) ────────────────────────────────────────────
# Whole-identifier $TASK_ID / ${TASK_ID} / $BENCHMARK / ${BENCHMARK}, not
# EVAL_-prefixed and not extended (so $EVAL_TASK_ID and $TASK_ID_STR are exempt).
# `$EVAL_TASK_ID` cannot match because after `$` the name begins with `E`, not the
# literal `TASK_ID`/`BENCHMARK`; the trailing boundary class rules out $TASK_ID_STR.

legacy_env_re := `\$\{?(TASK_ID|BENCHMARK)([^A-Za-z0-9_]|$)`

deny contains msg if {
	some tok in all_value_tokens
	regex.match(legacy_env_re, tok)
	msg := "references $TASK_ID or $BENCHMARK — must use $EVAL_TASK_ID / $EVAL_BENCHMARK (dockerfile_inspection legacy_env_var)"
}

# ── todo_or_fixme (red) ─────────────────────────────────────────────
# A `#` comment containing a STANDALONE TODO/FIXME/XXX token; a `FUTURE:` block is
# the sanctioned escape hatch and is exempt. buildkit exposes each comment as a
# `comment` instruction whose Value[0] is the text without the leading `#`.
# Boundaries are any non-[A-Za-z0-9] char (so `_` is a boundary too — matches the
# Rust split on `!char::is_alphanumeric`, which treats `_` as a separator). Hence
# `TODO_FIX` matches but `TODOS` / `myTODO` do not.
#
# PARITY NOTE: buildkit DROPS comments that trail the last instruction, so a
# `# TODO` as the final line(s) of a Dockerfile is invisible here though the Rust
# line-scanner would catch it. No Dockerfile in the tree ends on a comment, so the
# finding sets agree today; see the migration report.

todo_re := `(^|[^A-Za-z0-9])(TODO|FIXME|XXX)([^A-Za-z0-9]|$)`

deny contains msg if {
	some instr in input
	instr.Cmd == "comment"
	some tok in instr.Value
	not contains(tok, "FUTURE:")
	regex.match(todo_re, tok)
	msg := "Dockerfile comment contains TODO/FIXME/XXX — use FUTURE: for explicit future work (dockerfile_inspection todo_or_fixme)"
}

# ── silent_pip_fallback (red) ───────────────────────────────────────
# A RUN line with `pip install` / `pip3 install` AND (`2>/dev/null` OR `|| true`):
# stderr is swallowed / the failure is ignored, so a missing dep surfaces only as a
# silent reward=0 at grade time. Matches Rust's lowercase substring test.

deny contains msg if {
	some instr in input
	instr.Cmd == "run"
	some tok in instr.Value
	lc := lower(tok)
	pip_install(lc)
	contains(lc, "2>/dev/null")
	msg := "pip install with 2>/dev/null fallback — errors are swallowed, grade.py will silently fail (dockerfile_inspection silent_pip_fallback)"
}

deny contains msg if {
	some instr in input
	instr.Cmd == "run"
	some tok in instr.Value
	lc := lower(tok)
	pip_install(lc)
	contains(lc, "|| true")
	msg := "pip install with || true fallback — errors are swallowed, grade.py will silently fail (dockerfile_inspection silent_pip_fallback)"
}

pip_install(lc) if contains(lc, "pip install")

pip_install(lc) if contains(lc, "pip3 install")

# ── python_full_base (yellow) ───────────────────────────────────────
# FROM python:X without a -slim/-alpine/-dev variant. buildkit puts the image in
# Value[0] (any --platform flag is split into Flags), so no manual flag-stripping.

warn contains msg if {
	some instr in input
	instr.Cmd == "from"
	image := instr.Value[0]
	startswith(image, "python:")
	not contains(image, "-slim")
	not contains(image, "-alpine")
	not contains(image, "-dev")
	msg := sprintf("FROM %s without -slim suffix (dockerfile_inspection python_full_base, RULES.md 10a)", [image])
}

# ── stale_data_revision (yellow) ────────────────────────────────────
# eval.benchmark.data_revision value is empty / latest / main / master / HEAD.
# Label values are parsed quoted (`"main"`); strip the quotes before comparing.

stale_revision_values := {"", "latest", "main", "master", "HEAD"}

warn contains msg if {
	rev := data_revision_value
	stale_revision_values[rev]
	msg := sprintf("eval.benchmark.data_revision is %q — empty/latest/main/master/HEAD is a stale pointer (dockerfile_inspection stale_data_revision)", [rev])
}

# ── missing_data_revision_when_fetching_mutable_ref (yellow) ─────────
# A RUN step pulls from a mutable HuggingFace/GitHub ref AND the image has no
# data_revision label pinned to a non-mutable value → upstream can change the
# dataset under us. (buildkit strips heredoc bodies, matching Rust strip_heredocs;
# multi-line RUN continuations are already joined into the single RUN value.)

warn contains msg if {
	fetches_mutable_ref
	not has_pinned_data_revision
	msg := "Dockerfile fetches from a mutable ref (refs/convert/parquet, main, master) without pinning eval.benchmark.data_revision (dockerfile_inspection missing_data_revision_when_fetching_mutable_ref)"
}

fetches_mutable_ref if {
	some instr in input
	instr.Cmd == "run"
	some tok in instr.Value
	mutable_ref_token(tok)
}

mutable_ref_token(tok) if contains(tok, "refs/convert/parquet")

mutable_ref_token(tok) if contains(tok, "?revision=main")

mutable_ref_token(tok) if contains(tok, "?revision=master")

mutable_ref_token(tok) if {
	contains(tok, "raw.githubusercontent.com/")
	contains(tok, "/main/")
}

mutable_ref_token(tok) if {
	contains(tok, "raw.githubusercontent.com/")
	contains(tok, "/master/")
}

# A data_revision label whose (unquoted) value is non-empty and not a mutable
# pointer satisfies the pin (mirrors the Rust early-return on a good revision).
has_pinned_data_revision if {
	rev := data_revision_value
	rev != ""
	rev != "latest"
	rev != "main"
	rev != "master"
	rev != "HEAD"
}

# ── shared helpers ──────────────────────────────────────────────────

# First eval.benchmark.data_revision label value, quotes stripped. Mirrors the
# Rust helpers, which read the first occurrence.
data_revision_value := rev if {
	some p in label_pairs
	p.key == "eval.benchmark.data_revision"
	rev := unquote(p.value)
}

# Every string token across all instruction Values — the surface legacy_env_var
# scans (env values, run commands, args, labels, comments, …). Heredoc bodies are
# not present (buildkit strips them); Rust scans the raw text and would also see a
# heredoc-buried $TASK_ID, but the tree has none, so the two agree — see the
# parity note at the top of this file.
all_value_tokens contains tok if {
	some instr in input
	some v in instr.Value
	tok := v
}

# Strip a single pair of surrounding double quotes if present.
unquote(s) := trim(s, `"`)
