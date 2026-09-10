//! `eval-containers list` — what the fleet publishes, or what this checkout has.
//!
//! The published answer comes from one object: `evals/index:latest`, written by
//! containers/scripts/fleet-index.py — every artifact the registry holds, by
//! name, with its tags, plus the `eval.*` labels each component declares.
//! Reading it is three anonymous HTTP calls, so `list` answers for the whole
//! fleet from anywhere, with no images pulled and no credential.
//!
//! `--local` answers the other question — what does *this checkout* declare —
//! straight from `containers/*/*/Dockerfile`. Both feed the same renderer, so the
//! two views differ in source, never in shape.

use clap::{Args, Subcommand};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub target: ListTarget,
    /// List what this checkout declares, from `containers/*/*/Dockerfile`,
    /// instead of what the registry publishes.
    #[arg(long, global = true)]
    pub local: bool,
}

#[derive(Subcommand)]
pub enum ListTarget {
    /// List benchmarks
    Benchmarks,
    /// List agents
    Agents,
    /// List models
    Models,
    /// List eval images (benchmark + agent combinations)
    Evals {
        #[arg(long)]
        benchmark: Option<String>,
        #[arg(long)]
        agent: Option<String>,
    },
}

type Labels = BTreeMap<String, String>;

/// What a listing renders, from either source: the published index or this
/// checkout. Every field defaults, so a source missing one is a column short
/// rather than a hard failure.
#[derive(Deserialize, Default)]
pub struct Catalog {
    #[serde(default)]
    pub agents: Vec<String>,
    /// family → its baked task ids (empty for a shared-env family).
    #[serde(default)]
    pub families: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub meta: Meta,
    /// The (benchmark, agent) pairs that exist. Multiplying the two lists above
    /// is what once offered an agent against every benchmark in the fleet when it
    /// was published for one.
    #[serde(skip)]
    pub pairs: BTreeSet<(String, String)>,
}

#[derive(Deserialize, Default)]
pub struct Meta {
    #[serde(default)]
    pub benchmarks: BTreeMap<String, Labels>,
    #[serde(default)]
    pub agents: BTreeMap<String, Labels>,
    #[serde(default)]
    pub models: BTreeMap<String, Labels>,
}

pub fn execute(registry: &str, args: ListArgs) -> Result<(), String> {
    let catalog = if args.local {
        eprintln!("$ containers/*/*/Dockerfile");
        local_catalog(Path::new("containers"))?
    } else {
        eprintln!("$ {registry}/evals/index:latest");
        published(registry)?
    };
    for line in render(&catalog, &args.target) {
        println!("{line}");
    }
    Ok(())
}

// ── rendering: one shape, whichever source filled the catalog ────────────────

fn render(cat: &Catalog, target: &ListTarget) -> Vec<String> {
    match target {
        ListTarget::Benchmarks => {
            let mut out = vec![
                format!(
                    "{:<28} {:<48} {:>6} {:<10} INTERNET",
                    "BENCHMARK", "DESCRIPTION", "TASKS", "TYPE"
                ),
                "-".repeat(104),
            ];
            for (name, tasks) in &cat.families {
                let l = cat.meta.benchmarks.get(name);
                // A per-task family's size is the number of images it bakes; a
                // shared-env one declares its dataset size as a label.
                let size = if tasks.is_empty() {
                    label(l, "tasks")
                } else {
                    tasks.len().to_string()
                };
                out.push(format!(
                    "{:<28} {:<48} {:>6} {:<10} {}",
                    name,
                    truncate(&label(l, "description"), 48),
                    size,
                    label(l, "env"),
                    label(l, "internet"),
                ));
            }
            out
        }
        ListTarget::Agents => {
            let mut out = vec![
                format!("{:<28} {:<44} RUNTIME", "AGENT", "DESCRIPTION"),
                "-".repeat(82),
            ];
            for name in &cat.agents {
                let l = cat.meta.agents.get(name);
                out.push(format!(
                    "{:<28} {:<44} {}",
                    name,
                    truncate(&label(l, "description"), 44),
                    label(l, "runtime")
                ));
            }
            out
        }
        ListTarget::Models => {
            let mut out = vec![format!("{:<28} PROVIDER", "MODEL"), "-".repeat(44)];
            for (name, l) in &cat.meta.models {
                out.push(format!("{:<28} {}", name, label(Some(l), "provider")));
            }
            out
        }
        // One row per launchable pair, never one per baked task: swe-bench is one
        // benchmark with 500 tasks behind it, and 500 x 7 rows is not a listing.
        ListTarget::Evals { benchmark, agent } => {
            let mut out = vec![format!("{:<40} {:>6}", "EVAL", "TASKS"), "-".repeat(48)];
            for (name, a) in &cat.pairs {
                if benchmark.as_ref().is_some_and(|b| b != name)
                    || agent.as_ref().is_some_and(|want| want != a)
                {
                    continue;
                }
                let tasks = cat.families.get(name).map(Vec::as_slice).unwrap_or(&[]);
                let size = if tasks.is_empty() {
                    label(cat.meta.benchmarks.get(name), "tasks")
                } else {
                    tasks.len().to_string()
                };
                out.push(format!("{:<40} {:>6}", format!("{name}--{a}"), size));
            }
            out
        }
    }
}

fn label(labels: Option<&Labels>, key: &str) -> String {
    labels
        .and_then(|l| l.get(key))
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_else(|| "-".to_string())
}

/// Cut on a character, never a byte: an em dash in a description is three bytes,
/// and slicing into one panics the listing rather than shortening it.
fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let kept: String = s.chars().take(width.saturating_sub(1)).collect();
    format!("{kept}…")
}

// ── published: the fleet index ───────────────────────────────────────────────

/// The index as published: what exists, and what each component declares.
#[derive(Deserialize, Default)]
struct Index {
    /// image name → its tags.
    #[serde(default)]
    images: BTreeMap<String, Vec<String>>,
    /// `<kind>/<name>` → its `eval.*` labels.
    #[serde(default)]
    labels: BTreeMap<String, Labels>,
}

/// The tag a `run` would pull. An image published under any other tag is not
/// something anyone can launch, so it is not something to list as available:
/// a build whose per-arch images landed and whose manifest list was never
/// stitched leaves `latest-amd64` and no `latest`.
const LAUNCH_TAG: &str = "latest";

fn published(registry: &str) -> Result<Catalog, String> {
    Ok(from_index(fetch_index(registry)?))
}

/// The index, read as the same shape `--local` builds from the tree, so one
/// renderer serves both.
fn from_index(idx: Index) -> Catalog {
    let named = |kind: &str| -> Vec<String> {
        idx.labels
            .keys()
            .filter_map(|k| k.strip_prefix(kind).map(str::to_string))
            .collect()
    };
    let benchmarks = named("benchmarks/");
    // Longest first: `swe-bench-astropy__astropy-12907` is a task of `swe-bench`,
    // not a benchmark of its own.
    let mut by_length = benchmarks.clone();
    by_length.sort_by_key(|f| std::cmp::Reverse(f.len()));

    let mut cat = Catalog::default();
    let mut agents: BTreeMap<String, ()> = BTreeMap::new();
    for (name, tags) in &idx.images {
        let Some(pair) = name.strip_prefix("evals/") else {
            continue;
        };
        if !tags.iter().any(|t| t == LAUNCH_TAG) {
            continue;
        }
        let Some((benchmark, agent)) = pair.rsplit_once("--") else {
            continue;
        };
        if agent.ends_with("-standalone") {
            continue; // a variant of a pair already counted
        }
        let Some(family) = by_length
            .iter()
            .find(|f| benchmark == f.as_str() || benchmark.starts_with(&format!("{f}-")))
        else {
            continue;
        };
        agents.insert(agent.to_string(), ());
        cat.pairs.insert((family.clone(), agent.to_string()));
        let tasks = cat.families.entry(family.clone()).or_default();
        if benchmark != family {
            tasks.push(benchmark[family.len() + 1..].to_string());
        }
    }
    for tasks in cat.families.values_mut() {
        tasks.sort();
        tasks.dedup();
    }
    cat.agents = agents.into_keys().collect();
    cat.meta.benchmarks = idx
        .labels
        .iter()
        .filter_map(|(k, v)| Some((k.strip_prefix("benchmarks/")?.to_string(), strip(v))))
        .filter(|(name, _)| cat.families.contains_key(name))
        .collect();
    cat.meta.agents = idx
        .labels
        .iter()
        .filter_map(|(k, v)| Some((k.strip_prefix("agents/")?.to_string(), strip(v))))
        .filter(|(name, _)| cat.agents.contains(name))
        .collect();
    cat.meta.models = idx
        .labels
        .iter()
        .filter_map(|(k, v)| Some((k.strip_prefix("models/")?.to_string(), strip(v))))
        .filter(|(name, _)| idx.images.contains_key(&format!("models/{name}")))
        .collect();
    cat
}

/// `eval.benchmark.description` → `description`: the renderer asks for the short
/// key, and the tree and the index disagree only on the prefix.
fn strip(labels: &Labels) -> Labels {
    labels
        .iter()
        .filter_map(|(k, v)| Some((k.rsplit_once('.')?.1.to_string(), v.clone())))
        .collect()
}

fn fetch_index(registry: &str) -> Result<Index, String> {
    let (host, org) = registry
        .split_once('/')
        .ok_or_else(|| format!("registry {registry} is not <host>/<org>"))?;
    let repo = format!("{org}/evals/index");
    // Anonymous: the published packages are public, so `list` works with no
    // credential and no docker login.
    let token: TokenBody = curl_json(
        &format!("https://{host}/token?scope=repository:{repo}:pull&service={host}"),
        &[],
    )?;
    let auth = format!("Authorization: Bearer {}", token.token);
    let manifest: ManifestBody = curl_json(
        &format!("https://{host}/v2/{repo}/manifests/latest"),
        &[&auth, "Accept: application/vnd.oci.image.manifest.v1+json"],
    )?;
    let digest = manifest
        .layers
        .first()
        .map(|l| l.digest.clone())
        .ok_or_else(|| format!("{repo}:latest carries no index layer"))?;
    curl_json(
        &format!("https://{host}/v2/{repo}/blobs/{digest}"),
        &[&auth],
    )
}

#[derive(Deserialize)]
struct TokenBody {
    token: String,
}

#[derive(Deserialize)]
struct ManifestBody {
    #[serde(default)]
    layers: Vec<LayerRef>,
}

#[derive(Deserialize)]
struct LayerRef {
    digest: String,
}

fn curl_json<T: serde::de::DeserializeOwned>(url: &str, headers: &[&str]) -> Result<T, String> {
    let mut cmd = Command::new("curl");
    cmd.args(["-sfL", "--max-time", "60"]);
    for h in headers {
        cmd.args(["-H", h]);
    }
    let out = cmd
        .arg(url)
        .output()
        .map_err(|e| format!("failed to run curl: {e} (or use --local)"))?;
    if !out.status.success() {
        return Err(format!(
            "{url}: curl exited {} (or use --local)",
            out.status
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("{url}: {e}"))
}

// ── local: the checkout's own Dockerfiles ────────────────────────────────────

/// The same shape, filled from the tree. What a checkout can answer is what its
/// components declare — so a benchmark that has never been published lists here
/// and nowhere else, which is the point of `--local`.
fn local_catalog(containers: &Path) -> Result<Catalog, String> {
    if !containers.is_dir() {
        return Err(format!(
            "{}: not a checkout of the fleet — run --local from the repo root",
            containers.display()
        ));
    }
    let mut cat = Catalog::default();
    for (kind, prefix) in [
        ("benchmarks", "eval.benchmark"),
        ("agents", "eval.agent"),
        ("models", "eval.model"),
    ] {
        for (name, dir) in components(&containers.join(kind))? {
            let labels = std::fs::read_to_string(dir.join("Dockerfile"))
                .map(|d| labels_of(&d, prefix))
                .unwrap_or_default();
            match kind {
                "benchmarks" => {
                    // Task ids come from the benchmark's own tasks.txt when it has
                    // one; the ones that resolve theirs from a pinned upstream tree
                    // are a network call away, which --local is not.
                    let tasks = std::fs::read_to_string(dir.join("tasks.txt"))
                        .map(|t| task_ids(&t))
                        .unwrap_or_default();
                    cat.families.insert(name.clone(), tasks);
                    cat.meta.benchmarks.insert(name, labels);
                }
                "agents" => {
                    cat.agents.push(name.clone());
                    cat.meta.agents.insert(name, labels);
                }
                _ => {
                    cat.meta.models.insert(name, labels);
                }
            }
        }
    }
    // A checkout has published nothing, so what it can say is that each of its
    // benchmarks could pair with each of its agents.
    cat.pairs = cat
        .families
        .keys()
        .flat_map(|b| cat.agents.iter().map(move |a| (b.clone(), a.clone())))
        .collect();
    Ok(cat)
}

fn components(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        // `_chart` and friends are scaffolding, not components.
        if name.starts_with('_') || !e.path().is_dir() {
            continue;
        }
        out.push((name, e.path()));
    }
    out.sort();
    Ok(out)
}

/// `LABEL <prefix>.<key>="<value>"` → {key: value}. Matched on a LABEL line, so a
/// comment or a `RUN echo` mentioning a label cannot invent one (benchmark.rs).
fn labels_of(dockerfile: &str, prefix: &str) -> Labels {
    let mut out = Labels::new();
    for line in dockerfile.lines() {
        let Some(rest) = line.trim_start().strip_prefix("LABEL ") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix(&format!("{prefix}.")) else {
            continue;
        };
        let Some((key, value)) = rest.split_once("=\"") else {
            continue;
        };
        let Some(value) = value.strip_suffix('"') else {
            continue;
        };
        out.insert(key.trim().to_string(), value.to_string());
    }
    out
}

/// tasks.txt groups its ids under `#` headings; a heading read as a task id is a
/// row for an image nobody built.
fn task_ids(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        let mut cat: Catalog = serde_json::from_str(
            r#"{"agents": ["codex", "pi"],
                "families": {"aime": [], "swe-bench": ["astropy-1", "django-2"]},
                "meta": {"benchmarks": {"aime": {"description": "AIME", "tasks": "90",
                                                 "env": "shared-env", "internet": "false"}},
                         "agents": {"codex": {"description": "OpenAI Codex CLI",
                                              "runtime": "node"}},
                         "models": {"gpt-5": {"provider": "openai"}}}}"#,
        )
        .unwrap();
        // Every pair exists in this fixture; from_index() records only the pairs
        // the registry actually has, which is what evals_are_pairs pins.
        cat.pairs = cat
            .families
            .keys()
            .flat_map(|b| cat.agents.iter().map(move |a| (b.clone(), a.clone())))
            .collect();
        cat
    }

    #[test]
    fn a_per_task_family_is_one_row_sized_by_its_tasks() {
        let rows = render(&catalog(), &ListTarget::Benchmarks);
        let swe: Vec<_> = rows.iter().filter(|r| r.starts_with("swe-bench")).collect();
        assert_eq!(swe.len(), 1, "one row per benchmark, not per task");
        assert!(
            swe[0].contains(" 2 "),
            "sized by its baked tasks: {}",
            swe[0]
        );
    }

    #[test]
    fn a_shared_env_family_is_sized_by_its_dataset_label() {
        let rows = render(&catalog(), &ListTarget::Benchmarks);
        let aime = rows.iter().find(|r| r.starts_with("aime")).unwrap();
        assert!(aime.contains("AIME") && aime.contains("90") && aime.contains("shared-env"));
    }

    #[test]
    fn a_catalog_without_meta_still_lists() {
        // A catalog published before `meta` existed, or one whose labels a
        // component never declared: a column short, never a hard failure.
        let cat: Catalog =
            serde_json::from_str(r#"{"agents": ["codex"], "families": {"aime": []}}"#).unwrap();
        let rows = render(&cat, &ListTarget::Benchmarks);
        assert!(
            rows.iter()
                .any(|r| r.starts_with("aime") && r.contains('-'))
        );
    }

    #[test]
    fn evals_are_pairs_and_the_filters_narrow_them() {
        let all = render(
            &catalog(),
            &ListTarget::Evals {
                benchmark: None,
                agent: None,
            },
        );
        assert_eq!(all.len(), 2 + 2 * 2, "one row per family x agent");
        let one = render(
            &catalog(),
            &ListTarget::Evals {
                benchmark: Some("aime".into()),
                agent: Some("pi".into()),
            },
        );
        assert_eq!(one.len(), 3);
        assert!(one[2].starts_with("aime--pi"));
    }

    #[test]
    fn a_description_is_cut_on_a_character_not_a_byte() {
        // The `mock` agent's description carries an em dash; cutting mid-codepoint
        // panicked the whole listing.
        let s = "writes a fixed answer, calls nothing — a deterministic carrier";
        assert_eq!(truncate(s, 5), "writ…");
        assert_eq!(truncate(s, 40).chars().count(), 40);
        assert_eq!(truncate("short", 40), "short");
    }

    #[test]
    fn a_pair_the_index_does_not_list_is_not_offered() {
        // openhands was published for one benchmark and offered against
        // ninety-nine, because a listing multiplied the two axes instead of
        // reading the pairs.
        let idx: Index = serde_json::from_str(
            r#"{"images": {"evals/aime--codex": ["latest"],
                           "evals/aime--pi": ["latest"],
                           "evals/swe-bench-astropy-1--codex": ["latest"],
                           "evals/gaia--pi": ["latest-amd64"]},
                "labels": {"benchmarks/aime": {"eval.benchmark.tasks": "90"},
                           "benchmarks/swe-bench": {},
                           "benchmarks/gaia": {}}}"#,
        )
        .unwrap();
        let cat = from_index(idx);
        let rows = render(
            &cat,
            &ListTarget::Evals {
                benchmark: None,
                agent: None,
            },
        );
        let pairs: Vec<&String> = rows.iter().skip(2).collect();
        assert_eq!(pairs.len(), 3, "one row per published pair: {pairs:?}");
        assert!(pairs.iter().any(|r| r.starts_with("aime--codex")));
        assert!(pairs.iter().any(|r| r.starts_with("swe-bench--codex")));
        // gaia--pi has no `latest`, so nothing could pull it.
        assert!(!pairs.iter().any(|r| r.starts_with("gaia--")), "{pairs:?}");
    }

    #[test]
    fn labels_come_off_label_lines_only() {
        let d = "FROM x\n# eval.benchmark.description=\"a comment\"\n\
                 LABEL eval.benchmark.description=\"AIME\"\n\
                 LABEL eval.benchmark.env=\"per-task\"\n\
                 RUN echo eval.benchmark.tasks=\"9\"\n";
        let l = labels_of(d, "eval.benchmark");
        assert_eq!(l.get("description").unwrap(), "AIME");
        assert_eq!(l.get("env").unwrap(), "per-task");
        assert!(!l.contains_key("tasks"), "a RUN echo is not a label");
    }

    #[test]
    fn task_ids_skip_the_headings_that_group_them() {
        assert_eq!(
            task_ids("# django\nDjango__django-1\n\n# astropy\nastropy-2\n"),
            vec!["django__django-1", "astropy-2"]
        );
    }

    #[test]
    fn local_mode_needs_a_checkout_and_says_so() {
        let err = match local_catalog(Path::new("/nonexistent/containers")) {
            Err(e) => e,
            Ok(_) => panic!("listed a tree that is not there"),
        };
        assert!(err.contains("not a checkout"), "{err}");
    }
}
