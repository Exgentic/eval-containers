//! Compose tests: verify every compose.yaml parses without errors.
//!
//! Run: cargo test --test compose -- --ignored

use std::process::Command;

macro_rules! compose_test {
    ($name:ident, $file:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            let output = Command::new("docker")
                .args(["compose", "-f", $file, "config"])
                .output()
                .expect("failed to run docker compose config");
            assert!(output.status.success(),
                "compose config failed for {}: {}",
                $file, String::from_utf8_lossy(&output.stderr));
        }
    };
}

compose_test!(aime, "benchmarks/aime/compose.yaml");
compose_test!(simpleqa, "benchmarks/simpleqa/compose.yaml");
compose_test!(gpqa_diamond, "benchmarks/gpqa-diamond/compose.yaml");
compose_test!(math_500, "benchmarks/math-500/compose.yaml");
compose_test!(mmlu_pro, "benchmarks/mmlu-pro/compose.yaml");
compose_test!(humaneval, "benchmarks/humaneval/compose.yaml");
compose_test!(livecodebench, "benchmarks/livecodebench/compose.yaml");
compose_test!(usaco, "benchmarks/usaco/compose.yaml");
compose_test!(gaia, "benchmarks/gaia/compose.yaml");
compose_test!(bfcl, "benchmarks/bfcl/compose.yaml");
compose_test!(gdpval, "benchmarks/gdpval/compose.yaml");
compose_test!(appworld, "benchmarks/appworld/compose.yaml");
compose_test!(browsecomp, "benchmarks/browsecomp/compose.yaml");
compose_test!(kumo, "benchmarks/kumo/compose.yaml");
compose_test!(healthbench, "benchmarks/healthbench/compose.yaml");
compose_test!(hle, "benchmarks/hle/compose.yaml");
compose_test!(arc_agi, "benchmarks/arc-agi/compose.yaml");
compose_test!(mmmu, "benchmarks/mmmu/compose.yaml");
compose_test!(aider_polyglot, "benchmarks/aider-polyglot/compose.yaml");
compose_test!(mrcr, "benchmarks/mrcr/compose.yaml");
compose_test!(tau_bench, "benchmarks/tau-bench/compose.yaml");
compose_test!(osworld, "benchmarks/osworld/compose.yaml");
compose_test!(webarena, "benchmarks/webarena/compose.yaml");
compose_test!(swe_bench, "benchmarks/swe-bench/compose.yaml");
compose_test!(compilebench, "benchmarks/compilebench/compose.yaml");
compose_test!(terminal_bench, "benchmarks/terminal-bench/compose.yaml");
compose_test!(ifeval, "benchmarks/ifeval/compose.yaml");
compose_test!(mgsm, "benchmarks/mgsm/compose.yaml");
compose_test!(mbpp, "benchmarks/mbpp/compose.yaml");
