#!/usr/bin/env bats
#
# Trajectory rule UNIT tests — the per-PR gate.
#
# Framework-free port of the synthetic-input #[test]s in
# tests/sanity/task_inspection.rs (everything EXCEPT the fixture sweep
# `inspect_every_existing_fixture`, which is the release-only audit in
# task_inspection_audit.release.bats). One @test per Rust #[test], same
# names, same assertions, same rule ids.
#
# The extraction + rule engine lives in the committed jq program
# tests/sanity/task_inspection.jq. These tests drive it the way the Rust
# tests drive RULES / RUN_RULES: feed a synthetic task string or a
# synthetic run-summary and assert which rule ids fire.
#
# Split rationale (matches .github/workflows/test.yml): CI runs
# `cargo test --test task_inspection -- --skip inspect_every_existing_fixture`,
# i.e. exactly these unit tests on every PR; the fixture audit is excluded
# from the per-PR gate because it checks recorded-data quality, not PR
# soundness. The bats dispatcher (tests/run) makes the same split by file
# name: *.release.bats runs only under `tests/run --release`.

REPO="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
JQ_PROG="$REPO/tests/sanity/task_inspection.jq"

# ── jq drivers ──────────────────────────────────────────────────────────
# Every invocation passes --arg source "" because jq compiles all branches
# of the engine (the findings branch references $source), even when the
# selected mode never emits it.

# task_fires <task-text> : print the ids of task-half rules that fire.
task_fires() {
  printf '%s' "$1" | jq -rR --slurp --arg mode task_rules --arg source "" -f "$JQ_PROG"
}

# run_fires <summary-json> : print the ids of run-half rules that fire.
run_fires() {
  printf '%s' "$1" | jq -rR --slurp --arg mode run_rules --arg source "" -f "$JQ_PROG"
}

# refusal_is <text> : "true"/"false" — exposes content_is_refusal directly
# (drives heuristic_refusal_*).
refusal_is() {
  printf '%s' "$1" | jq -rR --slurp --arg mode refusal --arg source "" -f "$JQ_PROG"
}

# delegates_is <task> : "true"/"false" — exposes the file-delegation
# heuristic directly (drives heuristic_task_delegates_to_file_*).
delegates_is() {
  printf '%s' "$1" | jq -rR --slurp --arg mode delegates --arg source "" -f "$JQ_PROG"
}

# ── synthetic run summary (mirrors blank_summary() in the .rs) ──────────
#
# A "healthy" summary that fires zero run rules. Each run-rule test starts
# from this and flips exactly one field, exactly like the Rust tests do.
blank_summary() {
  cat <<'JSON'
{
  "n_rows": 5,
  "n_substantive_rows": 5,
  "n_failure_rows": 0,
  "last_substantive_status": "success",
  "any_assistant_content_nonempty": true,
  "total_tokens": 1000,
  "total_cost": 0.01,
  "max_consecutive_identical_prompts": 1,
  "any_error_message": "",
  "n_refusal_rows": 0,
  "n_max_tokens_rows": 0,
  "final_response_is_refusal": false,
  "task_delegates_to_file": false,
  "task_references_attachment": false,
  "fetch_required_but_no_tool_calls": false
}
JSON
}

# summary_with <field> <json-value-as-string> : blank_summary with one
# field overridden. Numbers/bools are passed through jq as raw JSON.
summary_with() {
  blank_summary | jq -c --argjson v "$2" --arg k "$1" '.[$k] = $v'
}

# Convenience: assert a rule id is present / absent in a fires() output.
assert_fires() { echo "$1" | grep -qx "$2"; }
refute_fires() { ! echo "$1" | grep -qx "$2"; }

# ── task-half rule unit tests ───────────────────────────────────────────

@test "rule_empty_fires_on_whitespace" {
  run task_fires "   $(printf '\n\t')  "
  [ "$status" -eq 0 ]
  assert_fires "$output" empty
}

@test "rule_env_leaked_fires_on_unresolved_eval_var" {
  run task_fires 'Solve task $EVAL_TASK_ID from benchmark ${EVAL_BENCHMARK}.'
  [ "$status" -eq 0 ]
  assert_fires "$output" env_leaked
}

@test "rule_template_leak_fires_on_placeholder" {
  run task_fires 'Solve this {NAME} problem: {TASK_PROMPT}'
  [ "$status" -eq 0 ]
  assert_fires "$output" template_leak
}

@test "rule_fetch_failed_fires_on_404" {
  run task_fires 'Task not found: 404 Not Found at huggingface.co/...'
  [ "$status" -eq 0 ]
  assert_fires "$output" fetch_failed
}

# ── run-half rule unit tests ────────────────────────────────────────────

@test "run_rule_refusal_final_response_fires" {
  run run_fires "$(summary_with final_response_is_refusal true)"
  [ "$status" -eq 0 ]
  assert_fires "$output" refusal_final_response
}

@test "run_rule_content_filter_fires_on_refusal_row" {
  run run_fires "$(summary_with n_refusal_rows 1)"
  [ "$status" -eq 0 ]
  assert_fires "$output" content_filter_refusal
}

@test "run_rule_max_tokens_truncation_fires" {
  run run_fires "$(summary_with n_max_tokens_rows 3)"
  [ "$status" -eq 0 ]
  assert_fires "$output" max_tokens_truncation
}

@test "run_rule_task_delegates_to_external_file_fires" {
  run run_fires "$(summary_with task_delegates_to_file true)"
  [ "$status" -eq 0 ]
  assert_fires "$output" task_delegates_to_external_file
}

@test "run_rule_last_substantive_failed_fires" {
  run run_fires "$(summary_with last_substantive_status '"failure"')"
  [ "$status" -eq 0 ]
  assert_fires "$output" last_substantive_row_failed
}

@test "run_rule_no_substantive_output_fires_on_empty" {
  run run_fires "$(summary_with any_assistant_content_nonempty false)"
  [ "$status" -eq 0 ]
  assert_fires "$output" no_substantive_output
}

@test "run_rule_cost_runaway_fires_above_cap" {
  run run_fires "$(summary_with total_cost 6.0)"
  [ "$status" -eq 0 ]
  assert_fires "$output" cost_runaway
}

@test "run_rule_token_runaway_fires_above_cap" {
  run run_fires "$(summary_with total_tokens 250000)"
  [ "$status" -eq 0 ]
  assert_fires "$output" token_runaway
}

@test "run_rule_retry_storm_fires_on_5" {
  run run_fires "$(summary_with max_consecutive_identical_prompts 5)"
  [ "$status" -eq 0 ]
  assert_fires "$output" retry_storm
}

@test "run_rule_context_overflow_fires_on_keyword" {
  run run_fires "$(summary_with any_error_message '"Error: context_length_exceeded (200000)"')"
  [ "$status" -eq 0 ]
  assert_fires "$output" context_overflow
}

# ── heuristic unit tests (positive AND negative, as in the .rs) ─────────

@test "heuristic_refusal_detects_azure_phrase" {
  [ "$(refusal_is "I'm sorry, but I cannot assist with that request.")" = true ]
  [ "$(refusal_is "I am happy to help with the task.")" = false ]
}

@test "heuristic_task_delegates_to_file_detects_short_pointer" {
  [ "$(delegates_is "Please read the task instructions at /app/task.txt and solve the problem.")" = true ]
  [ "$(delegates_is "Solve this aime problem: Let P(x) be a polynomial...")" = false ]
}

# ── clean-input tests (no rule should fire) ─────────────────────────────

@test "run_rule_clean_summary_produces_no_findings" {
  run run_fires "$(blank_summary)"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "clean_task_produces_no_findings" {
  local clean
  clean="Solve the following AIME problem. Print only the answer as a single integer.

Quadratic polynomials P(x) and Q(x) have leading coefficients 2 and -2..."
  run task_fires "$clean"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}
