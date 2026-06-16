# Unit tests for the second-tranche dockerfile_inspection rules (issue #114) —
# ports of the dockerfile_inspection.rs #[test]s plus their edge cases and the
# fail-loud injected violations. Run with `conftest verify --policy tests/policy/dockerfile`.
#
# These rules live in `package inspection`, so each fixture triggers ONLY these
# rules (not labels.rego's benchmark/agent LABEL contract). `eval.type="benchmark"`
# is the neutral type for exercising the type-agnostic rules: this package has no
# benchmark contract, so a benchmark fixture fires none of the agent/model rules.

package inspection

import rego.v1

# ── fixture helpers ─────────────────────────────────────────────────
run(cmd) := {"Cmd": "run", "Flags": [], "Value": [cmd]}

comment(text) := {"Cmd": "comment", "Flags": [], "Value": [text]}

from(image) := {"Cmd": "from", "Flags": [], "Value": [image]}

from_as(image, stage) := {"Cmd": "from", "Flags": [], "Value": [image, "AS", stage]}

arg(token) := {"Cmd": "arg", "Flags": [], "Value": [token]}

env(key, value) := {"Cmd": "env", "Flags": [], "Value": [key, value, "="]}

label(key, quoted_value) := {"Cmd": "label", "Flags": [], "Value": [key, quoted_value, "="]}

benchmark_type := label("eval.type", `"benchmark"`)

# ════════════════════════════════════════════════════════════════════
# missing_dock_type (red, domain-gated)
# (rule_missing_dock_type_fires)
# ════════════════════════════════════════════════════════════════════

test_missing_dock_type_fires if {
	bad := [from("alpine:3"), run("echo hi")]
	count(deny) == 1 with input as bad with data.params.category as "benchmarks"
}

test_missing_dock_type_ok_when_present if {
	count(deny) == 0 with input as [from("alpine:3"), benchmark_type]
		with data.params.category as "benchmarks"
}

# Out of the inspection domain (gateways/core) → not flagged, matching the Rust
# sweep, which never visits those roots.
test_missing_dock_type_skips_out_of_domain if {
	gw := [from("alpine:3"), label("gateway.kind", `"bifrost"`)]
	count(deny) == 0 with input as gw with data.params.category as "gateways"
}

# ════════════════════════════════════════════════════════════════════
# label_dir_mismatch (red)
# (rule_label_dir_mismatch_fires)
# ════════════════════════════════════════════════════════════════════

test_label_dir_mismatch_fires if {
	bad := [benchmark_type, label("eval.benchmark.name", `"other"`)]
	count(deny) == 1 with input as bad
		with data.params.dir as "mybench"
		with data.params.category as "benchmarks"
}

test_label_dir_match_ok if {
	ok := [benchmark_type, label("eval.benchmark.name", `"mybench"`)]
	count(deny) == 0 with input as ok
		with data.params.dir as "mybench"
		with data.params.category as "benchmarks"
}

# Absent name label → no finding (Rust returns "matches" when neither name exists).
test_label_dir_absent_name_ok if {
	count(deny) == 0 with input as [benchmark_type]
		with data.params.dir as "mybench"
		with data.params.category as "benchmarks"
}

# An agent's eval.agent.name is checked the same way.
test_label_dir_mismatch_agent_name if {
	bad := [label("eval.type", `"agent"`), label("eval.agent.name", `"wrong"`), arg("AGENT_VERSION=1")]
	count(deny) == 1 with input as bad
		with data.params.dir as "claude-code"
		with data.params.category as "agents"
}

# ════════════════════════════════════════════════════════════════════
# untagged_from (red)
# (rule_untagged_from_fires / rule_untagged_from_allows_scratch)
# ════════════════════════════════════════════════════════════════════

test_untagged_from_fires if {
	count(deny) == 1 with input as [from("ubuntu"), benchmark_type]
		with data.params.category as "benchmarks"
}

test_untagged_from_allows_scratch if {
	count(deny) == 0 with input as [from("scratch"), benchmark_type]
		with data.params.category as "benchmarks"
}

test_untagged_from_allows_tag if {
	count(deny) == 0 with input as [from("ubuntu:24.04"), benchmark_type]
		with data.params.category as "benchmarks"
}

test_untagged_from_allows_digest if {
	count(deny) == 0 with input as [from("ubuntu@sha256:abc"), benchmark_type]
		with data.params.category as "benchmarks"
}

# A tag on a registry path with slashes is read from the segment after the last `/`.
test_untagged_from_reads_tag_after_last_slash if {
	count(deny) == 0 with input as [from("ghcr.io/org/img:1.2"), benchmark_type]
		with data.params.category as "benchmarks"
}

# An untagged image on a slashed path IS flagged (tail after last `/` has no `:`).
test_untagged_from_slashed_path_untagged_fires if {
	count(deny) == 1 with input as [from("ghcr.io/org/img"), benchmark_type]
		with data.params.category as "benchmarks"
}

# `${REGISTRY}/…` (interpolated, in-repo) is exempt — the ref starts with `$`. (The
# global `ARG REGISTRY` keeps from_arg_not_global silent, isolating untagged_from.)
test_untagged_from_allows_interpolated if {
	ok := [arg("REGISTRY=ghcr.io/exgentic"), from("${REGISTRY}/core/agent-base-node:latest"), benchmark_type]
	count(deny) == 0 with input as ok with data.params.category as "benchmarks"
}

# ════════════════════════════════════════════════════════════════════
# todo_string_literal (red)
# ════════════════════════════════════════════════════════════════════

test_todo_string_literal_fires_in_run if {
	count(deny) == 1 with input as [run(`echo "TODO" > /task.txt`), benchmark_type]
		with data.params.category as "benchmarks"
}

test_todo_string_literal_single_quoted_fires if {
	count(deny) == 1 with input as [run(`echo 'TODO'`), benchmark_type]
		with data.params.category as "benchmarks"
}

# A `# TODO` comment is hygiene.rego's todo_or_fixme job, NOT this rule — the literal
# rule only inspects non-comment instructions, so it stays silent here.
test_todo_string_literal_ignores_comment if {
	count(deny) == 0 with input as [comment("TODO: later"), benchmark_type]
		with data.params.category as "benchmarks"
}

# ════════════════════════════════════════════════════════════════════
# from_arg_not_global (red) — ports of PR #130's bad/good cases
# (rule_from_arg_not_global_fires / _allows_global_decl / _allows_predefined_buildarg)
# ════════════════════════════════════════════════════════════════════

test_from_arg_not_global_fires if {
	# AGENT_VERSION declared AFTER the FROM that interpolates it (the plandex bug).
	bad := [
		from_as("up:v${AGENT_VERSION}", "src"),
		from("alpine:3"),
		arg("AGENT_VERSION=1.2.3"),
		benchmark_type,
	]
	count(deny) == 1 with input as bad with data.params.category as "benchmarks"
}

test_from_arg_not_global_allows_global_decl if {
	# Declared globally (before any FROM), then re-declared in-stage.
	ok := [
		arg("AGENT_VERSION=1.2.3"),
		from_as("up:v${AGENT_VERSION}", "src"),
		from("alpine:3"),
		arg("AGENT_VERSION=1.2.3"),
		benchmark_type,
	]
	count(deny) == 0 with input as ok with data.params.category as "benchmarks"
}

test_from_arg_not_global_allows_predefined_buildarg if {
	# Docker auto-provides TARGETARCH — a FROM may interpolate it (in --platform).
	ok := [
		{"Cmd": "from", "Flags": ["--platform=linux/${TARGETARCH}"], "Value": ["alpine:3"]},
		benchmark_type,
	]
	count(deny) == 0 with input as ok with data.params.category as "benchmarks"
}

# A non-global var interpolated inside a --platform flag is still flagged.
test_from_arg_not_global_flag_var_fires if {
	bad := [
		{"Cmd": "from", "Flags": ["--platform=linux/${MYARCH}"], "Value": ["alpine:3"]},
		benchmark_type,
	]
	count(deny) == 1 with input as bad with data.params.category as "benchmarks"
}

# The in-repo ${REGISTRY}/${REGISTRY_SUFFIX} pattern: both are global ARGs, so the
# FROM that expands them is fine (the real claude-code agent shape).
test_from_arg_global_registry_ok if {
	ok := [
		arg("REGISTRY=ghcr.io/exgentic"),
		arg("REGISTRY_SUFFIX=/"),
		from("${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest"),
		arg("AGENT_VERSION=2.1.104"),
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"claude-code"`),
	]
	count(deny) == 0 with input as ok
		with data.params.dir as "claude-code"
		with data.params.category as "agents"
}

# ════════════════════════════════════════════════════════════════════
# agent_missing_version_arg (red, type-gated)
# ════════════════════════════════════════════════════════════════════

test_agent_missing_version_arg_fires if {
	bad := [
		from("alpine:3"),
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"x"`),
		label("eval.agent.version", `"${AGENT_VERSION}"`),
	]
	# eval.type present → missing_dock_type silent; dir matches name → no mismatch;
	# only the missing ARG AGENT_VERSION fires.
	count(deny) == 1 with input as bad
		with data.params.dir as "x"
		with data.params.category as "agents"
}

test_agent_with_version_arg_ok if {
	ok := [
		from("alpine:3"),
		arg("AGENT_VERSION=1.2.3"),
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"x"`),
		label("eval.agent.version", `"${AGENT_VERSION}"`),
	]
	count(deny) == 0 with input as ok
		with data.params.dir as "x"
		with data.params.category as "agents"
}

# A non-agent (benchmark) is not subject to the ARG AGENT_VERSION requirement.
test_agent_version_arg_not_required_for_benchmark if {
	count(deny) == 0 with input as [from("alpine:3"), benchmark_type]
		with data.params.category as "benchmarks"
}

# ════════════════════════════════════════════════════════════════════
# model_missing_litellm_version_{label,default} (red, type-gated)
# ════════════════════════════════════════════════════════════════════

complete_litellm_model := [
	from("alpine:3"),
	label("eval.type", `"model"`),
	label("eval.model.litellm_version", `"main-v1.83.3-stable"`),
	env("EVAL_LITELLM_VERSION_DEFAULT", "main-v1.83.3-stable"),
]

test_model_complete_ok if {
	count(deny) == 0 with input as complete_litellm_model with data.params.dir as "gpt-5"
}

test_model_missing_litellm_label_fires if {
	bad := [
		from("alpine:3"),
		label("eval.type", `"model"`),
		env("EVAL_LITELLM_VERSION_DEFAULT", "main-v1.83.3-stable"),
	]
	count(deny) == 1 with input as bad with data.params.dir as "gpt-5"
}

test_model_missing_litellm_default_fires if {
	bad := [
		from("alpine:3"),
		label("eval.type", `"model"`),
		label("eval.model.litellm_version", `"main-v1.83.3-stable"`),
	]
	count(deny) == 1 with input as bad with data.params.dir as "gpt-5"
}

test_model_missing_both_fires_twice if {
	bad := [from("alpine:3"), label("eval.type", `"model"`)]
	count(deny) == 2 with input as bad with data.params.dir as "gpt-5"
}

# replay is the in-repo stub — exempt from both litellm requirements.
test_model_replay_exempt if {
	replay := [from("alpine:3"), label("eval.type", `"model"`)]
	count(deny) == 0 with input as replay with data.params.dir as "replay"
}

# Gateway-flavor models (LABEL gateway.kind=) are thin wrappers — exempt.
test_model_gateway_flavor_exempt if {
	gw := [from("alpine:3"), label("eval.type", `"model"`), label("gateway.kind", `"litellm"`)]
	count(deny) == 0 with input as gw with data.params.dir as "gpt-5.4--litellm"
}

# ════════════════════════════════════════════════════════════════════
# upstream_base_unpinned (yellow)
# ════════════════════════════════════════════════════════════════════

test_upstream_base_latest_warns if {
	count(warn) == 1 with input as [label("eval.benchmark.upstream_base", `"foo/bar:latest"`)]
}

test_upstream_base_no_tag_warns if {
	# No `:` and no `@` (the swe-bench-pro shape) → unpinned.
	count(warn) == 1 with input as [label("eval.benchmark.upstream_base", `"docker.io/jefzda/sweap-images (per-task)"`)]
}

test_upstream_base_pinned_ok if {
	# A trailing note keeps the value's `:`, so it reads as tagged (Rust contains(':')).
	count(warn) == 0 with input as [label("eval.benchmark.upstream_base", `"ubuntu:24.04 + upstream source"`)]
}

test_upstream_base_digest_ok if {
	count(warn) == 0 with input as [label("eval.benchmark.upstream_base", `"foo/bar@sha256:abc"`)]
}

# ════════════════════════════════════════════════════════════════════
# phantom_pip_uninstall (yellow, domain-gated)
# ════════════════════════════════════════════════════════════════════

test_phantom_pip_uninstall_warns if {
	bad := [run("rm -f /tmp/x.csv && pip uninstall -y pandas")]
	count(warn) == 1 with input as bad with data.params.category as "benchmarks"
}

# Uninstall combined with an install in the SAME RUN reclaims space — no finding.
test_phantom_pip_uninstall_same_run_ok if {
	ok := [run("pip install pyarrow && python extract.py && pip uninstall -y pyarrow")]
	count(warn) == 0 with input as ok with data.params.category as "benchmarks"
}

# Out of the inspection domain (a gateway's `pip uninstall … || true`) → no finding,
# matching the Rust sweep, which never visits gateways.
test_phantom_pip_uninstall_skips_out_of_domain if {
	gw := [run("/opt/gateway/venv/bin/pip uninstall -y prisma || true")]
	count(warn) == 0 with input as gw with data.params.category as "gateways"
}

# ════════════════════════════════════════════════════════════════════
# install_order_pip_before_apt (yellow, domain-gated)
# ════════════════════════════════════════════════════════════════════

test_install_order_pip_before_apt_warns if {
	bad := [
		run("pip install --no-cache-dir numpy==1.26.0"),
		run("apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*"),
	]
	count(warn) == 1 with input as bad with data.params.category as "benchmarks"
}

test_install_order_apt_before_pip_ok if {
	ok := [
		run("apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*"),
		run("pip install --no-cache-dir numpy==1.26.0"),
	]
	count(warn) == 0 with input as ok with data.params.category as "benchmarks"
}

# A single RUN doing both (apt then pip) is fine — equal index, not "pip before apt".
test_install_order_same_run_ok if {
	ok := [run("apt-get install -y curl && pip install --no-cache-dir numpy==1.26.0")]
	count(warn) == 0 with input as ok with data.params.category as "benchmarks"
}

# ════════════════════════════════════════════════════════════════════
# secret_in_arg_or_env (red) — credentials must never be baked (rule 8a)
# Fixtures are otherwise clean (tagged FROM + eval.type) so `count(deny)`
# isolates the secret rule.
# ════════════════════════════════════════════════════════════════════

# A credential baked as ENV is denied (the HF_TOKEN leak vector).
test_secret_env_denied if {
	bad := [from("python:3.12-slim"), benchmark_type, env("HF_TOKEN", "hf_x")]
	count(deny) == 1 with input as bad with data.params.category as "benchmarks"
}

# A credential passed as a build ARG is denied (persists in image history).
test_secret_arg_denied if {
	bad := [from("python:3.12-slim"), benchmark_type, arg("OPENAI_API_KEY")]
	count(deny) == 1 with input as bad with data.params.category as "benchmarks"
}

# The rule fires fleet-wide, not just the inspection domain (gateways/core too).
test_secret_env_denied_outside_inspection_domain if {
	bad := [from("python:3.12-slim"), env("AWS_SECRET_ACCESS_KEY", "x")]
	count(deny) == 1 with input as bad with data.params.category as "core"
}

# Version args that merely CONTAIN "key" must NOT trip it (`_KEY`, not bare `KEY`).
test_version_args_are_not_secrets if {
	ok := [from("python:3.12-slim"), benchmark_type, arg("PORTKEY_VERSION=1.15.2"), arg("AGENT_VERSION")]
	count(deny) == 0 with input as ok with data.params.category as "benchmarks"
}

# A gated fetch via the ephemeral secret mount is the SANCTIONED path — no ARG/ENV,
# so no finding (the RUN command mentioning HF_TOKEN is not an ARG/ENV instruction).
test_secret_mount_run_is_clean if {
	ok := [
		from("python:3.12-slim"), benchmark_type,
		run("--mount=type=secret,id=HF_TOKEN HF_TOKEN=$(cat /run/secrets/HF_TOKEN) curl ..."),
	]
	count(deny) == 0 with input as ok with data.params.category as "benchmarks"
}
