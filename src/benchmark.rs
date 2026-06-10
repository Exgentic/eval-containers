//! Benchmark metadata derived from a benchmark's `Dockerfile`.

/// A benchmark is *per-task* — one eval image baked per task (swe-bench-style) —
/// iff its `Dockerfile` declares `LABEL eval.benchmark.env="per-task"`. That
/// label is the single source of truth for per-task detection across the CLI
/// (build, run, oracle) and the chart's `perTask` value (benchmarks/RULES.md 24f).
/// Matched on a `LABEL` line so a comment or `RUN echo` mentioning the string
/// cannot false-positive.
pub fn is_per_task(dockerfile: &str) -> bool {
    dockerfile.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("LABEL ") && t.contains(r#"eval.benchmark.env="per-task""#)
    })
}

/// [`is_per_task`] for a benchmark by name — reads `benchmarks/<name>/Dockerfile`
/// (a missing file reads as shared-env, `false`).
pub fn is_per_task_by_name(name: &str) -> bool {
    std::fs::read_to_string(format!("benchmarks/{name}/Dockerfile"))
        .as_deref()
        .map(is_per_task)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_per_task_keys_off_the_label() {
        assert!(is_per_task(
            "FROM scratch\nLABEL eval.benchmark.env=\"per-task\"\n"
        ));
        assert!(!is_per_task(
            "FROM scratch\nLABEL eval.benchmark.env=\"shared\"\n"
        ));
        // A FROM/ARG mentioning EVAL_TASK_ID is NOT per-task without the label.
        assert!(!is_per_task("ARG EVAL_TASK_ID\nFROM x-${EVAL_TASK_ID}\n"));
        // A comment / RUN echo mentioning the label string must not false-positive.
        assert!(!is_per_task("# eval.benchmark.env=\"per-task\"\nFROM x\n"));
    }
}
