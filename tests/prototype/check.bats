#!/usr/bin/env bats
# Framework-free port of tests/sanity/check.rs (issue #114) — gauge the feel of
# the daemon-free "check" gate as bats instead of a Rust integration test.
# Each Rust #[test] becomes one bats @test, preserving the rule↔test pairing.
#
# Engine is plain shell (grep/find over files); bats only provides the test
# reporting/isolation. Throwaway/alongside — deletes nothing, changes no rule.
#
# Run: bats tests/prototype/check.bats

REPO="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"

# ── shared helpers ─────────────────────────────────────────────────────────
has_line()           { grep -qF -- "$2" "$1" 2>/dev/null; }
is_test_benchmark()  { has_line "$1/Dockerfile" 'LABEL eval.benchmark.env="test"'; }

# dir names under containers/<root>, skipping _* and dotfiles (mirrors check.rs)
catalog_dirs() {
  local d name
  for d in "$REPO/containers/$1"/*/; do
    name=$(basename "$d"); case $name in _*|.*) continue ;; esac
    echo "$name"
  done
}

BENCH_LABELS=(
  'LABEL eval.type="benchmark"'
  'LABEL eval.benchmark.name='
  'LABEL eval.benchmark.env='
  'LABEL eval.benchmark.tasks='
  'LABEL eval.benchmark.internet='
)
AGENT_LABELS=(
  'LABEL eval.type="agent"'
  'LABEL eval.agent.name='
  'LABEL eval.agent.version='
)
COMPOSE_MARKERS=( 'include:' 'compose/services.yaml' 'services:' '  runner:' 'BENCHMARK:' )

# ── step 6: structural validation ──────────────────────────────────────────
structural_issues() {
  local name dir df f lbl m req
  for name in $(catalog_dirs benchmarks); do
    dir="$REPO/containers/benchmarks/$name"; df="$dir/Dockerfile"
    [ -f "$df" ] || { echo "$name: no Dockerfile"; continue; }
    if is_test_benchmark "$dir"; then req="compose.yaml"; else req="container.Dockerfile compose.yaml"; fi
    for f in $req; do [ -f "$dir/$f" ] || echo "$name: no $f (rule 24 triple-mode contract)"; done
    for lbl in "${BENCH_LABELS[@]}"; do has_line "$df" "$lbl" || echo "$name: missing $lbl"; done
    if [ -f "$dir/compose.yaml" ]; then
      for m in "${COMPOSE_MARKERS[@]}"; do has_line "$dir/compose.yaml" "$m" || echo "$name: compose missing \`$m\`"; done
    fi
  done
  for name in $(catalog_dirs agents); do
    dir="$REPO/containers/agents/$name"; df="$dir/Dockerfile"
    [ -f "$df" ] || { echo "$name: no Dockerfile"; continue; }
    for lbl in "${AGENT_LABELS[@]}"; do has_line "$df" "$lbl" || echo "$name: missing $lbl"; done
    has_line "$df" 'LABEL eval.agent.version="latest"' && echo "$name: eval.agent.version is \`latest\` — must pin"
  done
  return 0
}

@test "structural validation (labels + triple-mode files)" {
  run structural_issues
  [ -z "$output" ] || { echo "structural issues:"; echo "$output"; false; }
}

# ── step 10: README count reconciliation ───────────────────────────────────
count_nontest_benchmarks() {
  local name n=0
  for name in $(catalog_dirs benchmarks); do
    is_test_benchmark "$REPO/containers/benchmarks/$name" && continue
    n=$((n + 1))
  done
  echo "$n"
}

@test "count reconciliation (README headline vs filesystem)" {
  local claim_b claim_a disk_b disk_a
  claim_b=$(grep -oE '[0-9]+ benchmarks' "$REPO/README.md" | head -1 | grep -oE '[0-9]+')
  claim_a=$(grep -oE '[0-9]+ agents'     "$REPO/README.md" | head -1 | grep -oE '[0-9]+')
  disk_b=$(count_nontest_benchmarks)
  disk_a=$(catalog_dirs agents | grep -c .)
  [ -n "$claim_b" ] || { echo "README has no '<N> benchmarks' claim"; false; }
  [ -n "$claim_a" ] || { echo "README has no '<N> agents' claim"; false; }
  [ "$claim_b" = "$disk_b" ] || { echo "README claims $claim_b benchmarks, filesystem has $disk_b"; false; }
  [ "$claim_a" = "$disk_a" ] || { echo "README claims $claim_a agents, filesystem has $disk_a"; false; }
}

# ── released benchmarks have a replay fixture ──────────────────────────────
released_benchmarks() {
  local name
  for name in $(catalog_dirs benchmarks); do
    has_line "$REPO/containers/benchmarks/$name/Dockerfile" 'LABEL eval.benchmark.released="true"' && echo "$name"
  done | sort
}
# filename: <benchmark>-<task>-<agent>.trajectory.jsonl — strip the trailing
# -<digits>-<agent>; greedy (.*) makes -<digits>- bind to the LAST group so
# multi-segment names like math-500 survive (matches check.rs's greedy scan).
fixture_benchmarks() {
  local f b
  for f in "$REPO"/tests/replay/fixtures/*.trajectory.jsonl; do
    [ -e "$f" ] || continue
    b=$(basename "$f" .trajectory.jsonl)
    sed -E 's/(.*)-[0-9]+-[a-z0-9-]+$/\1/' <<<"$b"
  done | sort -u
}

@test "released benchmarks each have >=1 replay fixture" {
  local fx b missing=""
  fx=$(fixture_benchmarks)
  for b in $(released_benchmarks); do
    grep -qx "$b" <<<"$fx" || missing="$missing $b"
  done
  [ -z "$missing" ] || { echo "released benchmarks with no fixture:$missing"; false; }
}

# ── steps 30, 31: README presence ──────────────────────────────────────────
@test "every (non-test) benchmark has README.md" {
  local name missing=""
  for name in $(catalog_dirs benchmarks); do
    is_test_benchmark "$REPO/containers/benchmarks/$name" && continue
    [ -f "$REPO/containers/benchmarks/$name/README.md" ] || missing="$missing $name"
  done
  [ -z "$missing" ] || { echo "benchmarks missing README.md:$missing"; false; }
}

@test "every agent has README.md" {
  local name missing=""
  for name in $(catalog_dirs agents); do
    [ -f "$REPO/containers/agents/$name/README.md" ] || missing="$missing $name"
  done
  [ -z "$missing" ] || { echo "agents missing README.md:$missing"; false; }
}

# ── OpenShift values overlay present ───────────────────────────────────────
@test "deploy/values-openshift.yaml present and sets anyuid-sa" {
  local v="$REPO/deploy/values-openshift.yaml"
  [ -f "$v" ] || { echo "missing $v"; false; }
  has_line "$v" 'serviceAccountName: anyuid-sa' || { echo "$v must set serviceAccountName: anyuid-sa"; false; }
  [ -f "$REPO/deploy/openshift-service-account.yaml" ] || { echo "missing deploy/openshift-service-account.yaml"; false; }
}

# ── #45: otelcol health gate consistent across the three modes ─────────────
@test "otelcol health gate consistent across modes (#45)" {
  local cfg svc job pc
  cfg="$REPO/containers/core/otel/config.yaml"
  svc="$REPO/containers/compose/services.yaml"
  job="$REPO/containers/benchmarks/_chart/templates/job.yaml"
  pc="$REPO/containers/core/process-compose/process-compose.yaml"
  has_line "$cfg" 'health_check:' && has_line "$cfg" 'extensions: [health_check]' \
    || { echo "otel config.yaml must enable+wire health_check (#45)"; false; }
  has_line "$svc" '13133' && has_line "$svc" 'condition: service_healthy' \
    || { echo "compose services.yaml must healthcheck otelcol :13133 + gate gateway service_healthy (#45)"; false; }
  # otelcol startupProbe on :13133 must appear before the gateway container
  local otelcol_block
  otelcol_block=$(sed '/- name: gateway/q' "$job")
  grep -q 'startupProbe:' <<<"$otelcol_block" && grep -q 'port: 13133' <<<"$otelcol_block" \
    || { echo "job.yaml otelcol sidecar must define startupProbe on :13133 (#45)"; false; }
  has_line "$pc" 'port: 13133' && has_line "$pc" 'condition: process_healthy' \
    || { echo "process-compose.yaml must probe otelcol :13133 + gate process_healthy (#45)"; false; }
}

# ── rule 12: stitched eval image launches the pipeline ─────────────────────
@test "eval image launches the pipeline (rule 12)" {
  local combo values runner_args
  combo="$REPO/containers/core/combination.Dockerfile"
  values="$REPO/containers/benchmarks/_chart/values.yaml"
  has_line "$combo" 'CMD ["/usr/local/bin/run"]' || { echo "combination.Dockerfile must set CMD [/usr/local/bin/run]"; false; }
  has_line "$combo" 'COPY --from=agent /run.sh' || { echo "combination.Dockerfile must COPY --from=agent /run.sh"; false; }
  runner_args=$(grep -E '^[[:space:]]*runnerArgs:' "$values" | head -1)
  [[ "$runner_args" == *"/usr/local/bin/run"* ]] || { echo "values.yaml runnerArgs must invoke /usr/local/bin/run"; false; }
}

# ── rule 7: agent env -i allow-list must not leak the task id ───────────────
@test "agent env -i allow-list excludes the task id (rule 7)" {
  local line
  line=$(grep -E 'gosu agent.*env -i' "$REPO/containers/core/process-compose/process-compose.yaml")
  [ -n "$line" ] || { echo "agent command (gosu agent env -i) not found"; false; }
  [[ "$line" != *"TASK_ID="* ]] || { echo "agent env -i allow-list leaks the task id:"; echo "$line"; false; }
}

# ── principle 9: version aligned across Cargo.toml and Chart.yaml ───────────
@test "repo version aligns across cli/Cargo.toml and Chart.yaml (principle 9)" {
  local cargo_ver chart_ver
  cargo_ver=$(grep -E '^version = "' "$REPO/cli/Cargo.toml" | head -1 | sed -E 's/version = "(.*)"/\1/')
  chart_ver=$(grep -E '^version:' "$REPO/containers/benchmarks/_chart/Chart.yaml" | head -1 | sed -E 's/version:[[:space:]]*//')
  [ -n "$cargo_ver" ] && [ -n "$chart_ver" ] || { echo "could not read both versions"; false; }
  [ "$cargo_ver" = "$chart_ver" ] || { echo "version drift: Cargo.toml ($cargo_ver) != Chart.yaml ($chart_ver)"; false; }
}
