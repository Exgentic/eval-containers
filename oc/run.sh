#!/usr/bin/env bash
# run.sh — build and run ONE eval on OpenShift.
#
# A dataset eval is one k8s Indexed Job (the dataset IS the sweep): each example
# is a completion index, k8s fans them out and caps concurrency. With --queue,
# Kueue admits the Job against a global quota; without it, --parallelism is the cap.
#
#   ./oc/run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost --dataset-size 500
#   ./oc/run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost --task 0   # single, debug
#
# Three standard tools: eval-containers (build) · helm (render) · oc (apply/watch).
#
# Flags: --benchmark --agent --model (required);
#   --dataset-size N   run the whole dataset as an Indexed Job (omit → single --task)
#   --task T           single-task debug run (default 0; ignored with --dataset-size)
#   --parallelism M    concurrency cap within the run (default: all at once)
#   --retry K          per-example retries (k8s ≥1.29; omit → no per-index retry)
#   --queue NAME       Kueue local-queue (omit → no Kueue, parallelism is the cap)
#   --eval-model S --namespace NS --registry URL --pvc NAME --repo-dir P
#   --rebuild          force rebuild even if the imagestream exists
#   --no-build         skip the build; --no-run build only, don't submit
#   --test             isolated -test imagestreams + runs-test/ path (prod untouched)
#   --rerun --watch --dry-run
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

BENCHMARK="" AGENT="" MODEL="" TASK="0" DATASET="" PARALLELISM="" RETRY="" QUEUE=""
EVAL_MODEL="" NAMESPACE="$NS_DEFAULT" REGISTRY="" PVC="eval-output-pvc" SWEEP_ID="" SUFFIX=""
DATASET_MODE=false NO_BUILD=false NO_RUN=false REBUILD=false TEST=false RERUN=false WATCH=false DRY_RUN=false
while [[ $# -gt 0 ]]; do case "$1" in
  --benchmark) BENCHMARK="$2"; shift 2;; --agent) AGENT="$2"; shift 2;;
  --model) MODEL="$2"; shift 2;; --task) TASK="$2"; shift 2;;
  --dataset) DATASET_MODE=true; shift;; --dataset-size) DATASET="$2"; DATASET_MODE=true; shift 2;;
  --parallelism) PARALLELISM="$2"; shift 2;; --retry) RETRY="$2"; shift 2;;
  --queue) QUEUE="$2"; shift 2;;
  --eval-model) EVAL_MODEL="$2"; shift 2;; --namespace) NAMESPACE="$2"; shift 2;;
  --registry) REGISTRY="$2"; shift 2;; --pvc) PVC="$2"; shift 2;;
  --repo-dir) REPO_DIR="$2"; shift 2;; --sweep-id) SWEEP_ID="$2"; shift 2;;
  --rebuild) REBUILD=true; shift;; --no-build) NO_BUILD=true; shift;;
  --no-run) NO_RUN=true; shift;; --test) TEST=true; shift;;
  --rerun) RERUN=true; shift;; --watch) WATCH=true; shift;; --dry-run) DRY_RUN=true; shift;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
  echo "error: --benchmark, --agent and --model are required" >&2; exit 1; }
log() { echo "[run] $*"; }

[[ -z "$REGISTRY" ]] && REGISTRY="$(oc_registry "$NAMESPACE")"
[[ -x "$REPO_DIR/target/release/eval-containers" ]] && PATH="$REPO_DIR/target/release:$PATH"
# --test: isolate everything behind a -test suffix so production is untouched.
RESULT_PREFIX="runs"; $TEST && { SUFFIX="-test"; RESULT_PREFIX="runs-test"; log "TEST MODE (-test imagestreams, $RESULT_PREFIX/ results)"; }

# ── 1. Build (CLI; skip if imagestream exists, unless --rebuild) ──────────────
if ! $NO_BUILD; then
  log "=== build ($BENCHMARK / $AGENT / $MODEL) ==="
  ISFLAG=(); [[ -n "$SUFFIX" ]] && ISFLAG=(--imagestream-suffix="$SUFFIX")
  build() { local label="$1" is="$2"; shift 2
    $DRY_RUN && { echo "[dry-run] eval-containers build $* --builder oc ${ISFLAG[*]:-}"; return; }
    ! $REBUILD && command oc get istag "${is}:latest" -n "$NAMESPACE" &>/dev/null && { log "skip $label (exists)"; return; }
    eval-containers build "$@" --builder oc ${ISFLAG[@]+"${ISFLAG[@]}"}; }
  ( cd "$REPO_DIR"
    build "bench" "$(flat "$BENCHMARK")$SUFFIX"        bench "$BENCHMARK"
    build "agent" "$(flat "$AGENT")$SUFFIX"            agent "$AGENT"
    build "model" "$(flat "$MODEL")$SUFFIX"            model "$MODEL"
    build "eval"  "$(flat "$BENCHMARK-$AGENT")$SUFFIX" eval "$BENCHMARK" --agent "$AGENT" --model "$MODEL" )
fi
$NO_RUN && { log "--no-run: built only, not submitting."; exit 0; }

# ── 2. Resolve dataset size, then render + apply ─────────────────────────────
# --dataset (whole dataset) with no explicit --dataset-size → read the count from
# the benchmark image's eval.benchmark.tasks label (set at build time). The image
# exists by now (built above), so this is the authoritative per-benchmark size —
# a grid of differently-sized benchmarks self-sizes without a flag.
if $DATASET_MODE && [[ -z "$DATASET" ]] && ! $DRY_RUN; then
  DATASET=$(command oc get istag "$(flat "$BENCHMARK")$SUFFIX:latest" -n "$NAMESPACE" \
    -o jsonpath='{.image.dockerImageMetadata.Config.Labels.eval\.benchmark\.tasks}' 2>/dev/null || true)
  [[ -z "$DATASET" ]] && { echo "error: could not read eval.benchmark.tasks label for $BENCHMARK; pass --dataset-size" >&2; exit 1; }
  log "dataset size for $BENCHMARK (from image label): $DATASET"
fi

if [[ -n "$DATASET" ]]; then JOB="${BENCHMARK}-${AGENT}${SUFFIX}"; SUB="${RESULT_PREFIX}/${BENCHMARK}/${AGENT}/${MODEL}";
else JOB="${BENCHMARK}-${AGENT}-task-${TASK}${SUFFIX}"; SUB="${RESULT_PREFIX}/${BENCHMARK}/${AGENT}/${MODEL}/${TASK}/${JOB}"; fi

[[ -z "$EVAL_MODEL" ]] && EVAL_MODEL="openai/azure/$(echo "$MODEL" | sed 's/--bifrost//;s/--litellm//;s/--portkey//')"
# flatImages=true → the chart composes flat ImageStream refs for the OC registry.
SET=(--set "benchmark=$BENCHMARK" --set "agent=$AGENT" --set "task=$TASK"
     --set "model=$MODEL" --set "gatewayImage=$MODEL" --set "evalModel=$EVAL_MODEL"
     --set "registry=$REGISTRY" --set "flatImages=true"
     --set "outputVolume.persistentVolumeClaim.claimName=$PVC" --set "outputSubPath=$SUB")
[[ -n "$SUFFIX"      ]] && SET+=(--set "imageSuffix=$SUFFIX" --set "nameSuffix=$SUFFIX")
[[ -n "$DATASET"     ]] && SET+=(--set "datasetSize=$DATASET")
[[ -n "$PARALLELISM" ]] && SET+=(--set "parallelism=$PARALLELISM")
[[ -n "$RETRY"       ]] && SET+=(--set "backoffLimitPerIndex=$RETRY")
[[ -n "$QUEUE"       ]] && SET+=(--set "queueName=$QUEUE")
[[ -n "$SWEEP_ID"    ]] && SET+=(--set "sweepId=$SWEEP_ID")

RENDER=$(helm template "$JOB" "$REPO_DIR/benchmarks/_chart" -f "$REPO_DIR/deploy/values-openshift.yaml" "${SET[@]}")
if $DRY_RUN; then echo "$RENDER"; exit 0; fi
$RERUN && command oc delete job "$JOB" -n "$NAMESPACE" --ignore-not-found >/dev/null
log "=== apply $JOB${DATASET:+ (Indexed, $DATASET examples${QUEUE:+, queue=$QUEUE})} ==="
printf '%s\n' "$RENDER" | command oc apply -n "$NAMESPACE" -f -

# ── 3. Watch (opt-in; with Kueue the Job may sit Suspended until admitted) ───
$WATCH || { log "submitted. status: ./oc/status.sh --benchmark $BENCHMARK"; exit 0; }
command oc wait --for=condition=complete --for=condition=failed "job/$JOB" -n "$NAMESPACE" --timeout=3600s || true
command oc get job "$JOB" -n "$NAMESPACE" -o jsonpath='Job {.metadata.name}: succeeded={.status.succeeded}/{.spec.completions} failed={.status.failed}{"\n"}'
