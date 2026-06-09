//! Benchmark metadata derived from a benchmark's `Dockerfile`.

/// A benchmark is *per-task* (one image per task, with `EVAL_TASK_ID` baked as a
/// build arg) iff its Dockerfile interpolates the task id into a `FROM` line, or
/// declares a default-less `ARG EVAL_TASK_ID`. A *runtime* `$EVAL_TASK_ID`
/// reference (resolved by `/eval-materialize-task`) does NOT count — that's the
/// shared-env JSONL path served from `all.jsonl`.
pub fn is_per_task(dockerfile: &str) -> bool {
    dockerfile.lines().any(|line| {
        let t = line.trim_start();
        if t.starts_with("FROM ") && (t.contains("${EVAL_TASK_ID}") || t.contains("$EVAL_TASK_ID"))
        {
            return true;
        }
        // `ARG EVAL_TASK_ID` on its own line with no `=<default>`.
        t.strip_prefix("ARG EVAL_TASK_ID")
            .is_some_and(|rest| rest.trim().is_empty())
    })
}
