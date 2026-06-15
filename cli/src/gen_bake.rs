//! `eval-containers gen-bake <artifact>` — scaffold a `docker-bake.hcl`
//! for a new artifact directory by parsing its `Dockerfile` and emitting
//! the file in the canonical shape from BAKE.md.
//!
//! Prevents future drift: contributors adding a new agent / benchmark
//! get a generated file that already passes the lint, instead of
//! hand-writing from the convention guide or copy-pasting a stale one.

use clap::Args;
use eval_containers::bake;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct GenBakeArgs {
    /// Artifact directory (e.g. `containers/agents/openhands`, `containers/benchmarks/aime`).
    pub artifact: String,
    /// Overwrite an existing `docker-bake.hcl` instead of erroring.
    #[arg(long)]
    pub force: bool,
}

pub fn execute(args: GenBakeArgs) -> Result<(), String> {
    let dir = PathBuf::from(&args.artifact);
    let dockerfile = dir.join("Dockerfile");
    if !dockerfile.exists() {
        return Err(format!(
            "{}/Dockerfile not found (gen-bake takes an artifact dir, not a category)",
            args.artifact
        ));
    }
    let (category, name) = split_artifact(&dir)?;
    let dockerfile_text = std::fs::read_to_string(&dockerfile)
        .map_err(|e| format!("read {}: {e}", dockerfile.display()))?;
    let deps = bake::dockerfile_in_repo_deps(&dockerfile_text);
    // Needs the HF_TOKEN build secret iff a `--mount=type=secret,id=HF_TOKEN` RUN
    // fetches gated data (rule 8a).
    let needs_hf_secret = dockerfile_text.contains("--mount=type=secret,id=HF_TOKEN");
    let content = render(&category, name, &deps, needs_hf_secret);

    let out = dir.join("docker-bake.hcl");
    if out.exists() && !args.force {
        return Err(format!(
            "{} already exists; pass --force to overwrite",
            out.display()
        ));
    }
    std::fs::write(&out, content).map_err(|e| format!("write {}: {e}", out.display()))?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

fn split_artifact(dir: &Path) -> Result<(String, &str), String> {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid artifact path: {}", dir.display()))?;
    let category = dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("artifact has no category parent: {}", dir.display()))?;
    if !["core", "agents", "benchmarks", "models", "gateways"].contains(&category) {
        return Err(format!(
            "unknown category `{category}` (expected one of core/agents/benchmarks/models/gateways)"
        ));
    }
    Ok((category.to_string(), name))
}

fn target_name_for(category: &str, name: &str) -> String {
    match category {
        "core" => name.replace('.', "_"),
        "agents" => format!("agent-{}", name),
        "benchmarks" => format!("benchmark-{}", name),
        "gateways" => format!("gateway-{}", name),
        "models" => format!("model-{}", name.replace('.', "_")),
        _ => name.to_string(),
    }
}

fn ref_to_target(image_ref: &str) -> String {
    let no_reg = &image_ref[bake::REGISTRY_PREFIX.len()..];
    let no_tag = no_reg.split(':').next().unwrap_or(no_reg);
    let mut parts = no_tag.splitn(2, '/');
    let cat = parts.next().unwrap_or("");
    let n = parts.next().unwrap_or("");
    target_name_for(cat, n)
}

fn render(category: &str, name: &str, deps: &[String], needs_hf_secret: bool) -> String {
    let target_name = target_name_for(category, name);
    let mut out = String::new();
    out.push_str(&format!("target \"{target_name}\" {{\n"));
    out.push_str(&format!("  context = \"containers/{category}/{name}\"\n"));
    if !deps.is_empty() {
        out.push_str("  contexts = {\n");
        for dep in deps {
            let key = dep.replace(bake::REGISTRY_PREFIX, "${REGISTRY}/");
            out.push_str(&format!(
                "    \"{key}\" = \"target:{}\"\n",
                ref_to_target(dep)
            ));
        }
        out.push_str("  }\n");
    }
    if needs_hf_secret {
        out.push_str("  secret = [\"id=HF_TOKEN,env=HF_TOKEN\"]\n");
    }
    out.push_str(&format!(
        "  tags = [\"${{REGISTRY}}/{category}/{name}:${{TAG}}\"]\n"
    ));
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_core() {
        let text = "FROM --platform=linux/amd64 python:3.12-slim\nRUN echo hi\n";
        assert_eq!(bake::dockerfile_in_repo_deps(text), Vec::<String>::new());
        let out = render("core", "agent-base-python", &[], false);
        assert!(out.contains("target \"agent-base-python\""));
        assert!(out.contains("context = \"containers/core/agent-base-python\""));
        assert!(out.contains("tags = [\"${REGISTRY}/core/agent-base-python:${TAG}\"]"));
        assert!(!out.contains("contexts"));
        assert!(!out.contains("HF_TOKEN"));
    }

    #[test]
    fn benchmark_with_hf_secret_and_deps() {
        // In-repo parameterized FROMs + a gated fetch (HF_TOKEN secret): guards the
        // contexts block AND that the token is emitted as a `secret`, not a build-arg.
        let text = "ARG REGISTRY=ghcr.io/exgentic\n\
            ARG REGISTRY_SUFFIX=/\n\
            FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match\n\
            FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest\n\
            RUN --mount=type=secret,id=HF_TOKEN HF_TOKEN=$(cat /run/secrets/HF_TOKEN) curl ...\n";
        let deps = bake::dockerfile_in_repo_deps(text);
        assert_eq!(
            deps,
            vec![
                "ghcr.io/exgentic/core/benchmark-base-hf".to_string(),
                "ghcr.io/exgentic/core/test-exact-match".to_string(),
            ]
        );
        let needs_hf_secret = text.contains("--mount=type=secret,id=HF_TOKEN");
        let out = render("benchmarks", "hle", &deps, needs_hf_secret);
        assert!(
            out.contains("\"${REGISTRY}/core/benchmark-base-hf\" = \"target:benchmark-base-hf\"")
        );
        assert!(
            out.contains("\"${REGISTRY}/core/test-exact-match\" = \"target:test-exact-match\"")
        );
        assert!(out.contains("secret = [\"id=HF_TOKEN,env=HF_TOKEN\"]"));
        assert!(!out.contains("args = { HF_TOKEN"));
        assert!(!out.contains("variable \"HF_TOKEN\""));
    }

    #[test]
    fn agent_with_in_repo_base() {
        let text = "ARG REGISTRY=ghcr.io/exgentic\n\
            ARG REGISTRY_SUFFIX=/\n\
            FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest\n";
        let deps = bake::dockerfile_in_repo_deps(text);
        let out = render("agents", "openhands", &deps, false);
        assert!(out.contains("target \"agent-openhands\""));
        assert!(
            out.contains("\"${REGISTRY}/core/agent-base-python\" = \"target:agent-base-python\"")
        );
        assert!(out.contains("tags = [\"${REGISTRY}/agents/openhands:${TAG}\"]"));
    }

    #[test]
    fn gateway_target_is_category_prefixed() {
        // Gateways follow <category>-<name> like agents/benchmarks, NOT a bare
        // name — a bare `litellm` would collide with core/litellm's target when
        // both bake files load in one invocation (RULES.md principle 15.a).
        let out = render("gateways", "litellm", &[], false);
        assert!(out.contains("target \"gateway-litellm\""));
        assert!(out.contains("context = \"containers/gateways/litellm\""));
        // A model FROMing its gateway resolves to the prefixed target.
        assert_eq!(
            ref_to_target("ghcr.io/exgentic/gateways/bifrost:latest"),
            "gateway-bifrost"
        );
    }
}
