# Eval-domain Dockerfile inspection — the SECOND tranche of the
# tests/sanity/dockerfile_inspection.rs port for issue #114.
#
# hygiene.rego already ported the first tranche (legacy_env_var, todo_or_fixme,
# silent_pip_fallback, python_full_base, stale_data_revision,
# missing_data_revision_when_fetching_mutable_ref). This file ports the remaining
# bespoke rules — the ones hadolint/gitleaks/trivy do NOT own and that the issue
# left for a follow-up — so dockerfile_inspection.rs can be deleted with parity.
#
# deny  (red):
#   - missing_dock_type     a benchmarks/agents/models Dockerfile with no LABEL eval.type
#   - label_dir_mismatch    eval.benchmark.name / eval.agent.name != the directory name
#   - untagged_from         an external FROM with no :tag and no @digest
#   - todo_string_literal   a non-comment instruction with a quoted "TODO" / 'TODO'
#   - from_arg_not_global   a FROM interpolates an ARG not declared in the global scope
#   - agent_missing_version_arg             agent image with no ARG AGENT_VERSION
#   - model_missing_litellm_version_label   model image with no LABEL eval.model.litellm_version
#   - model_missing_litellm_version_default model image with no ENV EVAL_LITELLM_VERSION_DEFAULT
# warn  (yellow):
#   - upstream_base_unpinned   eval.benchmark.upstream_base ends :latest, or has no tag/digest
#   - phantom_pip_uninstall    a RUN does `pip uninstall` with no `pip install` on it
#   - install_order_pip_before_apt   a pip-install RUN precedes an apt-get-install RUN
#
# KEPT IN RUST (NOT ported — procedural, cross-line lint that does not fit Rego's
# flat, break-less parse faithfully; see the migration report):
#   - unpinned_pip / unpinned_npm  (a token-walk after `pip install` / `npm i -g`
#     with stop-token `break` semantics — `\`, `&&`, `||`, `;`, `uninstall` — plus
#     the transient-pip-uninstall, `-r requirements`, `git+…@rev`/`#egg`, and
#     `.whl`/`.tgz`/`.tar.gz` exemptions). Faithfully expressing the "stop at the
#     first separator on this pip segment" left-to-right scan in Rego would be a
#     bad fit and risk silent divergence, so it stays in a slim Rust lint.
#   - hardcoded_secret  (owned by gitleaks, per the issue — not an eval-domain rule).
#
# SEPARATE PACKAGE: this lives in `package inspection`, not `main`, so its deny/warn
# sets are independent of the LABEL-contract denies in labels.rego (which would
# otherwise fire on every minimal benchmark/agent test fixture here) and can be
# unit-tested in isolation. conftest's run.sh sweeps with --all-namespaces, so both
# packages' findings are still aggregated over the tree. Shared parse helpers are
# reused from `main` via `import data.main` (no duplication).
#
# DOMAIN SCOPING: dockerfile_inspection.rs sweeps only containers/{benchmarks,agents,
# models}; the runner sweeps those plus gateways/core. Rules not already type-gated
# by `LABEL eval.type=` (missing_dock_type, label_dir_mismatch, untagged_from,
# todo_string_literal, from_arg_not_global, phantom_pip_uninstall,
# install_order_pip_before_apt) are gated on `in_scope` (category injected by
# run.sh) so the rego finding-set over 146 files equals the Rust set over 131.
# The type-gated rules (agent_*, model_*) and the label-only upstream_base_unpinned
# are inherently within that domain (only agents/models/benchmarks carry those
# labels), so they need no extra gate.

package inspection

import rego.v1

import data.main

# ── domain scope ────────────────────────────────────────────────────
inspection_categories := {"benchmarks", "agents", "models"}

in_scope if inspection_categories[data.params.category]

# secret_in_arg_or_env (red): a credential-shaped ARG/ENV bakes a secret into the
# image (history/config + inherited by children — the HF_TOKEN leak). Fires on EVERY
# artifact. `_KEY`/`APIKEY` not bare `KEY`, so `PORTKEY_VERSION` is spared. (rule 8a)
secret_name_pattern := `(?i)(TOKEN|SECRET|PASSWORD|PASSWD|CREDENTIAL|_KEY|APIKEY)`

deny contains msg if {
	some instr in input
	instr.Cmd == "arg"
	name := split(instr.Value[0], "=")[0]
	regex.match(secret_name_pattern, name)
	msg := sprintf("ARG %q is credential-shaped — build args persist in image history; never BAKE a secret. To USE one during the build, mount it with `--mount=type=secret`; otherwise pass it as a runtime env var (dockerfile_inspection secret_in_arg_or_env, benchmarks/RULES.md).", [name])
}

deny contains msg if {
	some instr in input
	instr.Cmd == "env"
	some idx, key in instr.Value
	idx % 3 == 0
	regex.match(secret_name_pattern, key)
	msg := sprintf("ENV %q is credential-shaped — it persists in the image config and is inherited by every child image; never BAKE a secret (mount it with `--mount=type=secret` to use one at build, otherwise a runtime env var) (dockerfile_inspection secret_in_arg_or_env, benchmarks/RULES.md).", [key])
}

# ── type classification (eval.type label, reusing main.label_value) ──
is_agent if main.label_value("eval.type") == `"agent"`

is_model if main.label_value("eval.type") == `"model"`

# ── missing_dock_type (red) ─────────────────────────────────────────
# A benchmarks/agents/models Dockerfile MUST declare `LABEL eval.type=`. (Rust:
# `!t.contains("LABEL eval.type=")`, scoped to those three roots — gateways carry
# only gateway.kind and are outside the Rust domain, so the scope gate keeps them
# from firing.)
deny contains msg if {
	in_scope
	not main.label_keys["eval.type"]
	msg := "Dockerfile is missing a LABEL eval.type= declaration (dockerfile_inspection missing_dock_type)"
}

# ── label_dir_mismatch (red) ────────────────────────────────────────
# eval.benchmark.name / eval.agent.name, when present, MUST equal the directory
# name (data.params.dir). (Rust label_name_matches_dir compares the first such
# label's unquoted value to the dir; absent → no finding.)
name_label_keys := {"eval.benchmark.name", "eval.agent.name"}

deny contains msg if {
	in_scope
	some p in main.label_pairs
	name_label_keys[p.key]
	main.unquote(p.value) != data.params.dir
	msg := sprintf(
		"%s=%s does not match directory %q (dockerfile_inspection label_dir_mismatch)",
		[p.key, p.value, data.params.dir],
	)
}

# ── untagged_from (red) ─────────────────────────────────────────────
# An external FROM with no `:tag` and no `@digest` is not reproducible. scratch and
# `$`-interpolated refs (incl. the in-repo `${REGISTRY}/…`) are exempt, matching
# Rust has_untagged_from. buildkit puts the image in Value[0] (any --platform flag
# is split into Flags), so the tag lives in the segment after the last `/`.
deny contains msg if {
	in_scope
	some instr in input
	instr.Cmd == "from"
	image := instr.Value[0]
	image != "scratch"
	not startswith(image, "$")
	segs := split(image, "/")
	tail := segs[count(segs) - 1]
	not contains(tail, ":")
	not contains(tail, "@")
	msg := sprintf("FROM %s without a tag — image is not reproducible (dockerfile_inspection untagged_from)", [image])
}

# ── todo_string_literal (red) ───────────────────────────────────────
# A NON-comment instruction containing the quoted literal "TODO" / 'TODO' (e.g. a
# RUN that writes the word "TODO" as task data → silent placeholder grading). The
# todo_or_fixme rule in hygiene.rego only inspects `#` comments and misses this.
# (Rust scans raw non-comment lines for `"TODO"` / `'TODO'`; buildkit keeps those
# quotes inside the instruction token, as the probe `echo "TODO"` confirmed.)
deny contains msg if {
	in_scope
	some instr in input
	instr.Cmd != "comment"
	some tok in instr.Value
	todo_literal(tok)
	msg := "Dockerfile writes the literal string \"TODO\" as task data (dockerfile_inspection todo_string_literal)"
}

todo_literal(tok) if contains(tok, `"TODO"`)

todo_literal(tok) if contains(tok, "'TODO'")

# ── from_arg_not_global (red) — port of dockerfile_inspection PR #130 ─
# A FROM may only interpolate an ARG declared in the GLOBAL scope (before the first
# FROM). An ARG declared inside a stage is invisible to FROM — Docker expands it to
# empty and silently corrupts the image ref (the plandex bug). A name a FROM
# expands that is neither a global ARG nor a Docker predefined build arg is flagged.
predefined_build_args := {
	"TARGETPLATFORM", "TARGETOS", "TARGETARCH", "TARGETVARIANT",
	"BUILDPLATFORM", "BUILDOS", "BUILDARCH", "BUILDVARIANT",
}

# Index of the first FROM in the (ordered) instruction array.
first_from_index := idx if {
	idxs := [i | some i, instr in input; instr.Cmd == "from"]
	idx := min(idxs)
}

# Global ARG names = ARG instructions positioned before the first FROM. The ARG
# token is `NAME` or `NAME=default`; take the part before any `=`.
global_arg_names contains name if {
	some i, instr in input
	instr.Cmd == "arg"
	i < first_from_index
	name := split(instr.Value[0], "=")[0]
}

# Variable names interpolated as $VAR / ${VAR} (the `${VAR:-x}` modifier is not
# rescanned: the name class stops at `:`). Scans both the image (Value) and the FROM
# flags (e.g. `--platform=linux/${TARGETARCH}`), matching Rust's raw-line scan.
from_interpolated_vars(instr) := {name |
	some field in array.concat(instr.Value, instr.Flags)
	m := regex.find_all_string_submatch_n(`\$\{?([A-Za-z0-9_]+)`, field, -1)
	some pair in m
	name := pair[1]
}

deny contains msg if {
	in_scope
	some instr in input
	instr.Cmd == "from"
	some var in from_interpolated_vars(instr)
	not predefined_build_args[var]
	not global_arg_names[var]
	msg := sprintf(
		"FROM interpolates `$%s`, an ARG not declared in the global scope (before any FROM) — Docker expands it to empty and silently corrupts the image tag (dockerfile_inspection from_arg_not_global)",
		[var],
	)
}

# ── agent_missing_version_arg (red, type-gated) ─────────────────────
# An agent image pins its upstream CLI version as `ARG AGENT_VERSION` (RULES.md 9 —
# version is a build arg, driving both the install and the eval.agent.version
# label). Type-gated by is_agent, so it never fires outside the agent domain.
has_agent_version_arg if {
	some instr in input
	instr.Cmd == "arg"
	startswith(instr.Value[0], "AGENT_VERSION")
}

deny contains msg if {
	is_agent
	not has_agent_version_arg
	msg := "agent Dockerfile is missing `ARG AGENT_VERSION` (dockerfile_inspection agent_missing_version_arg, RULES.md 9)"
}

# ── model_missing_litellm_version_{label,default} (red, type-gated) ─
# A model image that actually wraps the LiteLLM proxy MUST record the version on
# both axes: LABEL eval.model.litellm_version= and ENV EVAL_LITELLM_VERSION_DEFAULT=.
# Exempt: models/replay (its own minimal server, no litellm) and gateway-flavor
# models (LABEL gateway.kind=, thin wrappers over the gateways). Type-gated by
# is_model.
is_replay_model if data.params.dir == "replay"

is_gateway_flavor_model if main.label_keys["gateway.kind"]

litellm_model if {
	is_model
	not is_replay_model
	not is_gateway_flavor_model
}

# ENV keys present anywhere (ENV packs as flat (key, value, "=") triples like LABEL).
env_keys contains key if {
	some instr in input
	instr.Cmd == "env"
	some idx, key in instr.Value
	idx % 3 == 0
}

deny contains msg if {
	litellm_model
	not main.label_keys["eval.model.litellm_version"]
	msg := "model Dockerfile is missing LABEL eval.model.litellm_version (dockerfile_inspection model_missing_litellm_version_label)"
}

deny contains msg if {
	litellm_model
	not env_keys["EVAL_LITELLM_VERSION_DEFAULT"]
	msg := "model Dockerfile is missing ENV EVAL_LITELLM_VERSION_DEFAULT (dockerfile_inspection model_missing_litellm_version_default)"
}

# ── upstream_base_unpinned (yellow) ─────────────────────────────────
# `LABEL eval.benchmark.upstream_base` ending in `:latest`, or with no tag/digest at
# all, is supply-chain debt (benchmarks/RULES.md 21b). Only benchmarks carry the
# label, so the rule is inherently within the inspection domain. (Rust splits the
# value on quotes and takes the first segment; main.unquote is the parse-time
# equivalent — a trailing parenthetical note keeps the value's `:`, so it reads as
# "tagged", exactly as the Rust contains(':') branch does.)
warn contains msg if {
	some p in main.label_pairs
	p.key == "eval.benchmark.upstream_base"
	val := main.unquote(p.value)
	upstream_base_is_unpinned(val)
	msg := sprintf("eval.benchmark.upstream_base %q is unpinned (:latest or no tag/digest) — supply-chain debt (dockerfile_inspection upstream_base_unpinned, benchmarks/RULES.md 21b)", [val])
}

upstream_base_is_unpinned(val) if endswith(val, ":latest")

upstream_base_is_unpinned(val) if {
	not contains(val, ":")
	not contains(val, "@")
}

# ── phantom_pip_uninstall (yellow, domain-gated) ────────────────────
# A RUN that does `pip uninstall` with no `pip install` on the same instruction
# reclaims no space — the install layer still holds the files (RULES.md 10b).
# Domain-gated: a `pip uninstall … || true` in a gateway build is legitimate and
# out of the Rust domain.
phantom_run(instr) if {
	instr.Cmd == "run"
	some tok in instr.Value
	lc := lower(tok)
	contains(lc, "pip uninstall")
	not contains(lc, "pip install")
}

warn contains msg if {
	in_scope
	some instr in input
	phantom_run(instr)
	msg := "pip uninstall in its own RUN layer reclaims no space (dockerfile_inspection phantom_pip_uninstall, RULES.md 10b) — combine with the install"
}

# ── install_order_pip_before_apt (yellow, domain-gated) ─────────────
# A pip-install RUN that precedes an apt-get-install RUN is a layer-cache smell: pip
# layers churn, apt is stable — run apt first so its layer caches. (Rust tracks the
# first pip-install RUN index and fires on a later apt-get-install RUN; instructions
# in `input` preserve file order, so the index comparison is direct. A single RUN
# doing both has equal indices and does not fire — same as Rust.)
run_pip_install_indices contains i if {
	some i, instr in input
	instr.Cmd == "run"
	some tok in instr.Value
	contains(lower(tok), "pip install")
}

first_pip_install_index := min(run_pip_install_indices)

warn contains msg if {
	in_scope
	some j, instr in input
	instr.Cmd == "run"
	some tok in instr.Value
	contains(lower(tok), "apt-get install")
	j > first_pip_install_index
	msg := "pip install runs before apt-get install — reverse the order so the stable apt layer can cache (dockerfile_inspection install_order_pip_before_apt)"
}
