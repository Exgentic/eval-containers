# Unit tests for the LABEL contract — ports of the check.rs structural cases.
# Run with `conftest verify --policy tests/policy/dockerfile`.

package main

import rego.v1

# Helper: a label instruction object for one key/value (value quoted as buildkit
# emits it).
label(key, quoted_value) := {
	"Cmd": "label",
	"Flags": [],
	"Value": [key, quoted_value, "="],
}

complete_benchmark := [
	label("eval.type", `"benchmark"`),
	label("eval.benchmark.name", `"x"`),
	label("eval.benchmark.env", `"shared-env"`),
	label("eval.benchmark.tasks", `"10"`),
	label("eval.benchmark.internet", `"false"`),
]

complete_agent := [
	label("eval.type", `"agent"`),
	label("eval.agent.name", `"x"`),
	label("eval.agent.version", `"1.2.3"`),
]

# ── classification ──────────────────────────────────────────────────

test_classifies_benchmark if {
	is_benchmark with input as complete_benchmark
}

test_classifies_agent if {
	is_agent with input as complete_agent
}

# A non-benchmark/agent eval.type (e.g. core-base) triggers neither contract.
test_core_base_is_neither if {
	core := [label("eval.type", `"core-base"`)]
	not is_benchmark with input as core
	not is_agent with input as core
}

# ── benchmark contract ──────────────────────────────────────────────

test_complete_benchmark_passes if {
	count(deny) == 0 with input as complete_benchmark
}

test_benchmark_missing_internet_denies if {
	missing := [
		label("eval.type", `"benchmark"`),
		label("eval.benchmark.name", `"x"`),
		label("eval.benchmark.env", `"shared-env"`),
		label("eval.benchmark.tasks", `"10"`),
	]
	count(deny) == 1 with input as missing
}

# ── agent contract ──────────────────────────────────────────────────

test_complete_agent_passes if {
	count(deny) == 0 with input as complete_agent
}

test_agent_missing_version_denies if {
	missing := [
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"x"`),
	]
	count(deny) == 1 with input as missing
}

test_agent_version_latest_denies if {
	bad := [
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"x"`),
		label("eval.agent.version", `"latest"`),
	]
	count(deny) == 1 with input as bad
}

# Build-arg version (the real agents) is fine — only the literal "latest" is bad.
test_agent_version_buildarg_passes if {
	ok := [
		label("eval.type", `"agent"`),
		label("eval.agent.name", `"x"`),
		label("eval.agent.version", `"${AGENT_VERSION}"`),
	]
	count(deny) == 0 with input as ok
}

# ── multi-label-per-line packing ────────────────────────────────────
# buildkit packs `LABEL a="x" b="y"` as one instruction with flat triples; the
# extractor must still see both keys.

test_multi_label_line_is_unpacked if {
	multi := [{
		"Cmd": "label",
		"Flags": [],
		"Value": [
			"eval.type", `"benchmark"`, "=",
			"eval.benchmark.name", `"x"`, "=",
			"eval.benchmark.env", `"shared-env"`, "=",
			"eval.benchmark.tasks", `"10"`, "=",
			"eval.benchmark.internet", `"false"`, "=",
		],
	}]
	count(deny) == 0 with input as multi
}
