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

/// [`is_per_task`] for a benchmark by name — reads `containers/benchmarks/<name>/Dockerfile`
/// **relative to the current directory**, so it answers only inside a checkout of
/// the fleet; anywhere else every benchmark reads as shared-env (`false`).
///
/// That is fine for `build`/`oracle`, which build from the catalog and cannot run
/// without it. It is NOT fine for a surface that is meant to work from a published
/// artifact alone: `run --mode job` used to call this and silently rendered
/// `evals/<b>--<a>` for a per-task benchmark whenever it ran outside the repo. The
/// chart now resolves that itself from its committed `per-task.json` (rule 24h),
/// derived from these same labels and CI-checked against them.
pub fn is_per_task_by_name(name: &str) -> bool {
    std::fs::read_to_string(format!("containers/benchmarks/{name}/Dockerfile"))
        .as_deref()
        .map(is_per_task)
        .unwrap_or(false)
}

/// The one agent a native-harness benchmark pairs with (`containers/agents/native`).
pub const NATIVE: &str = "native";

/// `LABEL eval.benchmark.agent="native"`: the benchmark's own harness is its
/// only agent (benchmarks/RULES.md 12a). LABEL-line matching, as [`is_per_task`].
pub fn is_native(dockerfile: &str) -> bool {
    dockerfile.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("LABEL ") && t.contains(r#"eval.benchmark.agent="native""#)
    })
}

/// [`is_native`] by name, read like [`is_per_task_by_name`]; `None` outside a
/// checkout, where the label cannot be read.
pub fn is_native_by_name(name: &str) -> Option<bool> {
    std::fs::read_to_string(format!("containers/benchmarks/{name}/Dockerfile"))
        .ok()
        .map(|df| is_native(&df))
}

/// The agent `benchmark` runs with, given the one asked for (`None` = the
/// surface's default): native pairs only with native (benchmarks/RULES.md 12b);
/// outside a checkout the request passes through.
pub fn agent_for(benchmark: &str, requested: Option<&str>) -> Result<Option<String>, String> {
    pair(benchmark, is_native_by_name(benchmark), requested)
}

fn pair(
    benchmark: &str,
    native: Option<bool>,
    requested: Option<&str>,
) -> Result<Option<String>, String> {
    match (native, requested) {
        (Some(true), None | Some(NATIVE)) => Ok(Some(NATIVE.to_string())),
        (Some(true), Some(other)) => Err(format!(
            "{benchmark} is a native-harness benchmark: its own harness is its only agent \
             (evals/{benchmark}--native), so it cannot run with --agent {other}"
        )),
        (Some(false), Some(NATIVE)) => Err(format!(
            "{benchmark} does not declare a native harness (LABEL eval.benchmark.agent=\"native\"), \
             so there is no evals/{benchmark}--native to run — pick an agent"
        )),
        (_, requested) => Ok(requested.map(str::to_string)),
    }
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

    #[test]
    fn is_native_keys_off_the_label() {
        assert!(is_native(
            "FROM scratch\nLABEL eval.benchmark.agent=\"native\"\n"
        ));
        assert!(!is_native(
            "FROM scratch\nLABEL eval.benchmark.env=\"shared-env\"\n"
        ));
        assert!(!is_native("# eval.benchmark.agent=\"native\"\nFROM x\n"));
    }

    #[test]
    fn native_pairs_only_with_native() {
        assert_eq!(pair("walle", Some(true), None), Ok(Some("native".into())));
        assert_eq!(
            pair("walle", Some(true), Some("native")),
            Ok(Some("native".into()))
        );
        assert!(pair("walle", Some(true), Some("codex")).is_err());
        assert!(pair("aime", Some(false), Some("native")).is_err());
        assert_eq!(
            pair("aime", Some(false), Some("codex")),
            Ok(Some("codex".into()))
        );
        assert_eq!(pair("aime", Some(false), None), Ok(None));
        // Outside a checkout the label is unreadable: the request passes through.
        assert_eq!(pair("walle", None, None), Ok(None));
        assert_eq!(
            pair("aime", None, Some("native")),
            Ok(Some("native".into()))
        );
    }
}
