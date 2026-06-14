# Unit tests for the eval-specific hygiene rules — ports of the
# dockerfile_inspection.rs #[test]s (and their edge cases). Run with
# `conftest verify --policy tests/policy/dockerfile`.

package main

import rego.v1

run(cmd) := {"Cmd": "run", "Flags": [], "Value": [cmd]}

comment(text) := {"Cmd": "comment", "Flags": [], "Value": [text]}

from(image) := {"Cmd": "from", "Flags": [], "Value": [image]}

# ── legacy_env_var ──────────────────────────────────────────────────
# (rule_legacy_env_var_fires / rule_legacy_env_var_allows_dock_prefix)

test_legacy_env_var_fires_on_task_id if {
	count(deny) == 1 with input as [run("echo $TASK_ID")]
}

test_legacy_env_var_fires_on_braced_benchmark if {
	count(deny) == 1 with input as [run("echo ${BENCHMARK}")]
}

test_legacy_env_var_allows_eval_prefix if {
	count(deny) == 0 with input as [run("echo $EVAL_TASK_ID")]
}

test_legacy_env_var_allows_extended_identifier if {
	# $TASK_ID_STR / $BENCHMARKS are different identifiers, not the legacy var.
	count(deny) == 0 with input as [run("echo $TASK_ID_STR and $BENCHMARKS")]
}

# ── todo_or_fixme ───────────────────────────────────────────────────
# (rule_todo_or_fixme_fires / rule_todo_allows_future_block)

test_todo_comment_fires if {
	count(deny) == 1 with input as [comment("TODO: fix this")]
}

test_fixme_comment_fires if {
	count(deny) == 1 with input as [comment("FIXME later")]
}

test_future_block_allowed if {
	count(deny) == 0 with input as [comment("FUTURE: consider swapping to alpine")]
}

test_todo_substring_not_flagged if {
	# TODOS / myTODO are not standalone TODO tokens.
	count(deny) == 0 with input as [comment("TODOS and myTODO are fine")]
}

# ── silent_pip_fallback ─────────────────────────────────────────────

test_silent_pip_devnull_fires if {
	count(deny) == 1 with input as [run("pip install numpy 2>/dev/null || pip3 install numpy")]
}

test_silent_pip_or_true_fires if {
	count(deny) == 1 with input as [run("pip install scipy || true")]
}

test_clean_pip_passes if {
	count(deny) == 0 with input as [run("pip install --no-cache-dir numpy==1.26.0")]
}

# ── python_full_base (warn) ─────────────────────────────────────────

test_python_full_base_warns if {
	count(warn) == 1 with input as [from("python:3.11")]
}

test_python_slim_ok if {
	count(warn) == 0 with input as [from("python:3.11-slim")]
}

test_python_alpine_ok if {
	count(warn) == 0 with input as [from("python:3.11-alpine")]
}

test_python_dev_ok if {
	count(warn) == 0 with input as [from("python:3.11-dev")]
}

# ── stale_data_revision (warn) ──────────────────────────────────────

test_data_revision_main_warns if {
	count(warn) == 1 with input as [{
		"Cmd": "label",
		"Flags": [],
		"Value": ["eval.benchmark.data_revision", `"main"`, "="],
	}]
}

test_data_revision_pinned_ok if {
	count(warn) == 0 with input as [{
		"Cmd": "label",
		"Flags": [],
		"Value": ["eval.benchmark.data_revision", `"13f9e12f"`, "="],
	}]
}

# ── missing_data_revision_when_fetching_mutable_ref (warn) ──────────

test_mutable_fetch_without_pin_warns if {
	count(warn) == 1 with input as [run("curl https://hf.co/d/resolve/refs/convert/parquet/x.parquet")]
}

test_mutable_fetch_with_pin_ok if {
	count(warn) == 0 with input as [
		run("curl https://hf.co/d/resolve/refs/convert/parquet/x.parquet"),
		{
			"Cmd": "label",
			"Flags": [],
			"Value": ["eval.benchmark.data_revision", `"abc123"`, "="],
		},
	]
}

# A pin that is itself a mutable pointer does NOT satisfy the requirement.
test_mutable_fetch_with_stale_pin_still_warns if {
	# Two warnings: the missing-pin rule AND stale_data_revision both fire.
	count(warn) == 2 with input as [
		run("curl https://hf.co/d/resolve/refs/convert/parquet/x.parquet"),
		{
			"Cmd": "label",
			"Flags": [],
			"Value": ["eval.benchmark.data_revision", `"main"`, "="],
		},
	]
}
