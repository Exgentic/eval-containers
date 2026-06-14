# LABEL contract — conftest/OPA port of tests/sanity/check.rs::structural_validation
# (the Dockerfile-label half) for issue #114.
#
# A Dockerfile is classified by its `LABEL eval.type=` value (per the issue:
# "identify type by the eval.type label"). Across the whole tree this is
# equivalent to classifying by directory — every containers/benchmarks/*
# Dockerfile carries eval.type="benchmark" and every containers/agents/* one
# carries eval.type="agent"; models/core/gateways use other values (or none) and
# are intentionally outside this contract, exactly as the Rust sweep scoped it to
# the benchmarks/ and agents/ dirs only.
#
# Contract (verbatim from check.rs REQUIRED_*_LABELS):
#   benchmark: eval.type="benchmark" AND keys eval.benchmark.{name,env,tasks,internet}
#   agent:     eval.type="agent"     AND keys eval.agent.{name,version},
#              with eval.agent.version != "latest"
#
# Like the Rust check, the four eval.benchmark.* requirements assert only that the
# KEY is present (any value); only eval.type and eval.agent.version constrain the
# value. conftest exposes the parsed Dockerfile as a flat array of instruction
# objects in `input`, and prints the offending file path on every result line, so
# the messages below need not repeat it. LABEL values arrive as quoted tokens
# (`"benchmark"`), so the value comparisons include the surrounding quotes.

package main

import rego.v1

# ── LABEL extraction ────────────────────────────────────────────────
#
# buildkit packs every `LABEL` instruction as a flat array of (key, value, "=")
# triples — one instruction may declare several labels:
#   LABEL a="x" b="y"  ->  Value = ["a", "\"x\"", "=", "b", "\"y\"", "="]
# label_pairs walks those triples and yields {key, value} with the value token
# exactly as parsed (quotes retained).

label_pairs contains {"key": key, "value": value} if {
	some instr in input
	instr.Cmd == "label"
	some idx, key in instr.Value
	idx % 3 == 0 # keys sit at positions 0, 3, 6, … (key, value, "=")
	value := instr.Value[idx + 1]
}

# Set of label keys present anywhere in the file.
label_keys contains key if {
	some p in label_pairs
	key := p.key
}

# Value for a given label key (quoted, as parsed). Used for the value-constrained
# labels (eval.type, eval.agent.version).
label_value(key) := value if {
	some p in label_pairs
	p.key == key
	value := p.value
}

# ── Type classification (by the eval.type label) ────────────────────

is_benchmark if label_value("eval.type") == `"benchmark"`

is_agent if label_value("eval.type") == `"agent"`

# ── Benchmark contract ──────────────────────────────────────────────

required_benchmark_keys := {
	"eval.benchmark.name",
	"eval.benchmark.env",
	"eval.benchmark.tasks",
	"eval.benchmark.internet",
}

deny contains msg if {
	is_benchmark
	some key in required_benchmark_keys
	not label_keys[key]
	msg := sprintf("benchmark Dockerfile is missing LABEL %s= (check.rs structural contract)", [key])
}

# ── Agent contract ──────────────────────────────────────────────────

required_agent_keys := {"eval.agent.name", "eval.agent.version"}

deny contains msg if {
	is_agent
	some key in required_agent_keys
	not label_keys[key]
	msg := sprintf("agent Dockerfile is missing LABEL %s= (check.rs structural contract)", [key])
}

deny contains msg if {
	is_agent
	label_value("eval.agent.version") == `"latest"`
	msg := "eval.agent.version is `latest` — must pin (check.rs structural contract)"
}
