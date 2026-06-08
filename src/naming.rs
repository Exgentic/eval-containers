//! Image-name and bake-target conventions — the single source of truth for
//! how the CLI maps a (benchmark, agent, model) axis onto registry image refs
//! and `docker buildx bake` target names.
//!
//! These were previously inlined as `format!` strings scattered across
//! `build.rs`, `run.rs`, `inspect.rs`, and `images.rs`. Centralizing them here
//! makes the conventions testable (see the unit tests below) and keeps every
//! call site in agreement — the contract that benchmarks/agents/models are
//! built and run against (src/RULES.md, benchmarks/RULES.md rule 24).

/// `{registry}/benchmarks/<name>:<tag>` — the per-benchmark base image.
pub fn benchmark_image(registry: &str, benchmark: &str, tag: &str) -> String {
    format!("{registry}/benchmarks/{benchmark}:{tag}")
}

/// `{registry}/benchmarks/<name>-<task>:<tag>` — a per-task benchmark variant
/// (swe-bench-style), built outside bake's static graph (BAKE.md).
pub fn benchmark_task_image(registry: &str, benchmark: &str, task_id: &str, tag: &str) -> String {
    format!("{registry}/benchmarks/{benchmark}-{task_id}:{tag}")
}

/// `{registry}/agents/<name>:<tag>` — the per-agent image.
pub fn agent_image(registry: &str, agent: &str, tag: &str) -> String {
    format!("{registry}/agents/{agent}:{tag}")
}

/// `{registry}/models/<model>:<tag>` — the per-model gateway image.
pub fn model_image(registry: &str, model: &str, tag: &str) -> String {
    format!("{registry}/models/{model}:{tag}")
}

/// `{registry}/evals/<benchmark>--<agent>:<tag>` — the combined eval image.
/// The `--` separator is load-bearing: the OpenShift flattener collapses it to
/// a single `-` for imagestream names (see [`flatten_imagestream`]).
pub fn eval_image(registry: &str, benchmark: &str, agent: &str, tag: &str) -> String {
    format!("{registry}/evals/{benchmark}--{agent}:{tag}")
}

/// `{registry}/compose/<name>:latest` — a benchmark's published compose file.
pub fn compose_artifact(registry: &str, benchmark: &str) -> String {
    format!("{registry}/compose/{benchmark}:latest")
}

/// Bake target for an agent: `agent-<name>`.
pub fn agent_bake_target(agent: &str) -> String {
    format!("agent-{agent}")
}

/// Bake target for a benchmark: `benchmark-<name>`.
pub fn benchmark_bake_target(benchmark: &str) -> String {
    format!("benchmark-{benchmark}")
}

/// Bake target for a model: `model-<name>`, with `.` → `_` because HCL target
/// names can't contain dots (e.g. `gpt-5.4` → `model-gpt-5_4`).
pub fn model_bake_target(model: &str) -> String {
    format!("model-{}", model.replace('.', "_"))
}

/// Nested image repo path → OpenShift imagestream name (single segment).
/// `core`/`gateways` keep their prefix (`core/otel` → `core-otel`); the
/// per-eval categories drop it (`benchmarks/aime` → `aime`,
/// `evals/aime--codex` → `aime-codex`); dots and `--` collapse to `-`.
pub fn flatten_imagestream(repo: &str) -> String {
    let (cat, rest) = repo.split_once('/').unwrap_or(("", repo));
    let name = rest.to_lowercase().replace('.', "-").replace("--", "-");
    match cat {
        "core" | "gateways" => format!("{cat}-{name}"),
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const REG: &str = "quay.io/eval-containers";

    #[test]
    fn eval_image_uses_double_dash_separator() {
        assert_eq!(
            eval_image(REG, "aime", "claude-code", "2.5.0"),
            "quay.io/eval-containers/evals/aime--claude-code:2.5.0"
        );
    }

    #[test]
    fn category_images_are_namespaced() {
        assert_eq!(
            benchmark_image(REG, "aime", "latest"),
            "quay.io/eval-containers/benchmarks/aime:latest"
        );
        assert_eq!(
            agent_image(REG, "codex", "latest"),
            "quay.io/eval-containers/agents/codex:latest"
        );
        assert_eq!(
            model_image(REG, "gpt-5.4--bifrost", "latest"),
            "quay.io/eval-containers/models/gpt-5.4--bifrost:latest"
        );
    }

    #[test]
    fn per_task_variant_appends_task_id() {
        assert_eq!(
            benchmark_task_image(REG, "swe-bench", "42", "latest"),
            "quay.io/eval-containers/benchmarks/swe-bench-42:latest"
        );
    }

    #[test]
    fn model_bake_target_replaces_dots() {
        assert_eq!(model_bake_target("gpt-5.4"), "model-gpt-5_4");
        assert_eq!(
            model_bake_target("gpt-5.4--bifrost"),
            "model-gpt-5_4--bifrost"
        );
        assert_eq!(model_bake_target("replay"), "model-replay");
    }

    #[test]
    fn flatten_drops_eval_categories_keeps_core() {
        assert_eq!(flatten_imagestream("core/otel"), "core-otel");
        assert_eq!(flatten_imagestream("gateways/bifrost"), "gateways-bifrost");
        assert_eq!(flatten_imagestream("benchmarks/aime"), "aime");
        assert_eq!(flatten_imagestream("evals/aime--codex"), "aime-codex");
        assert_eq!(flatten_imagestream("models/gpt-5.4"), "gpt-5-4");
    }
}
