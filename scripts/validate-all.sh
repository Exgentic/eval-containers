#!/usr/bin/env bash
# Validate every benchmark and agent on disk against the structural contract.
# Does NOT build anything — purely grep/ls based. Runs in seconds.
#
# Exits 0 on clean, 1 on any failure.

set -uo pipefail

cd "$(dirname "$0")/.."

bench_issues=""
agent_issues=""
bench_bad=0
agent_bad=0

issue() {
  echo "  $1: $2"
  return 0
}

# ---------- benchmarks ----------
for d in benchmarks/*/; do
  name=$(basename "$d")
  case "$name" in _*|RULES.md|TEMPLATE.md) continue ;; esac

  df="${d}Dockerfile"
  cf="${d}compose.yaml"
  bad=0

  if [ ! -f "$df" ]; then
    bench_issues+="  $name: no Dockerfile"$'\n'
    bench_bad=$((bench_bad+1))
    continue
  fi
  [ -f "$cf" ] || { bench_issues+="  $name: no compose.yaml"$'\n'; bad=1; }

  grep -q 'LABEL dock.type="benchmark"'    "$df" || { bench_issues+="  $name: missing dock.type label"$'\n'; bad=1; }
  grep -q 'LABEL dock.benchmark.name='     "$df" || { bench_issues+="  $name: missing dock.benchmark.name"$'\n'; bad=1; }
  grep -q 'LABEL dock.benchmark.env='      "$df" || { bench_issues+="  $name: missing dock.benchmark.env"$'\n'; bad=1; }
  grep -q 'LABEL dock.benchmark.tasks='    "$df" || { bench_issues+="  $name: missing dock.benchmark.tasks"$'\n'; bad=1; }
  grep -q 'LABEL dock.benchmark.internet=' "$df" || { bench_issues+="  $name: missing dock.benchmark.internet"$'\n'; bad=1; }

  if [ -f "$cf" ]; then
    grep -q '^services:'     "$cf" || { bench_issues+="  $name: compose no services:"$'\n'; bad=1; }
    grep -q '^  model:'      "$cf" || { bench_issues+="  $name: compose no model service"$'\n'; bad=1; }
    grep -q '^  eval:'       "$cf" || { bench_issues+="  $name: compose no eval service"$'\n'; bad=1; }
    grep -q '^networks:'     "$cf" || { bench_issues+="  $name: compose no networks"$'\n'; bad=1; }
    grep -q 'compose/services.yaml' "$cf" || { bench_issues+="  $name: compose doesn't extend compose/services.yaml"$'\n'; bad=1; }
  fi

  [ $bad -eq 1 ] && bench_bad=$((bench_bad+1))
done

# ---------- agents ----------
for d in agents/*/; do
  name=$(basename "$d")
  case "$name" in _*|RULES.md|TEMPLATE.md) continue ;; esac

  df="${d}Dockerfile"
  bad=0

  if [ ! -f "$df" ]; then
    agent_issues+="  $name: no Dockerfile"$'\n'
    agent_bad=$((agent_bad+1))
    continue
  fi

  grep -q 'LABEL dock.type="agent"'     "$df" || { agent_issues+="  $name: missing dock.type label"$'\n'; bad=1; }
  grep -q 'LABEL dock.agent.name='      "$df" || { agent_issues+="  $name: missing dock.agent.name"$'\n'; bad=1; }
  grep -q 'LABEL dock.agent.version='   "$df" || { agent_issues+="  $name: missing dock.agent.version"$'\n'; bad=1; }
  grep -q 'LABEL dock.agent.version="latest"' "$df" && { agent_issues+="  $name: version is :latest (must pin)"$'\n'; bad=1; }

  [ $bad -eq 1 ] && agent_bad=$((agent_bad+1))
done

# ---------- report ----------
bench_count=$(find benchmarks -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
agent_count=$(find agents -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')

echo "==================================="
echo "  Dock structural validation"
echo "==================================="
echo "Benchmarks checked: $bench_count"
echo "Agents checked:     $agent_count"
echo

if [ $bench_bad -eq 0 ]; then
  echo "OK: all benchmarks pass"
else
  echo "FAIL: $bench_bad benchmarks with issues"
  printf '%s' "$bench_issues"
fi
echo

if [ $agent_bad -eq 0 ]; then
  echo "OK: all agents pass"
else
  echo "FAIL: $agent_bad agents with issues"
  printf '%s' "$agent_issues"
fi

[ $bench_bad -eq 0 ] && [ $agent_bad -eq 0 ] || exit 1
exit 0
