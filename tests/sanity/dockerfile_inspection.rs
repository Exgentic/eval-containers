//! Dockerfile health inspection: does every benchmark and agent
//! Dockerfile follow the rules and conventions?
//!
//! See tests/DOCKERFILE.md for the signal catalog and the manual
//! audit procedure. This file is the mechanical layer: a data-driven
//! rule catalog applied to the raw text of every Dockerfile under
//! `benchmarks/*/Dockerfile` and `agents/*/Dockerfile`.
//!
//! Same pattern as tests/task_inspection.rs — rules as a const array
//! of (id, severity, why, test fn) rows. Adding a rule is one line.
//! Rule IDs match the signal catalog in DOCKERFILE.md so the doc and
//! the code can't drift.
//!
//! Run: cargo test --test dockerfile_inspection

use std::fs;
use std::path::{Path, PathBuf};

// ─── Rule types ────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Severity {
    Red,
    Yellow,
}

/// A Dockerfile rule. Some rules need the directory name (to check
/// label drift against it), so every test function receives a
/// (dockerfile_text, directory_name) pair.
struct Rule {
    id: &'static str,
    severity: Severity,
    why: &'static str,
    test: fn(&str, &str) -> bool,
}

impl Rule {
    const fn red(id: &'static str, why: &'static str, test: fn(&str, &str) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Red,
            why,
            test,
        }
    }
    const fn yellow(id: &'static str, why: &'static str, test: fn(&str, &str) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Yellow,
            why,
            test,
        }
    }
}

// ─── Small rule helpers ────────────────────────────────────────────

fn contains_hardcoded_api_key(t: &str) -> bool {
    // Real-looking keys only — avoid false positives on documentation
    // that mentions `sk-proxy` as a placeholder or `sk-...` in examples.
    for line in t.lines() {
        // Skip comment lines
        if line.trim_start().starts_with('#') {
            continue;
        }
        // OpenAI: sk-[A-Za-z0-9]{40+}
        if let Some(i) = line.find("sk-") {
            let tail = &line[i + 3..];
            let alnum: String = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if alnum.len() >= 40 {
                return true;
            }
        }
        // GitHub PAT: ghp_[A-Za-z0-9]{36}
        if line.contains("ghp_")
            && let Some(i) = line.find("ghp_")
        {
            let tail = &line[i + 4..];
            let alnum: String = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if alnum.len() >= 36 {
                return true;
            }
        }
        // AWS: AKIA[0-9A-Z]{16}
        if line.contains("AKIA")
            && let Some(i) = line.find("AKIA")
        {
            let tail = &line[i + 4..];
            let caps: String = tail
                .chars()
                .take_while(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
                .collect();
            if caps.len() == 16 {
                return true;
            }
        }
    }
    false
}

fn has_unpinned_pip(t: &str) -> bool {
    // Helper: is this package uninstalled later in the same file? If so,
    // it's a transient build-time tool (e.g. `pyarrow` used to extract
    // dataset parquet at build time, then uninstalled). Transient build
    // tools are allowed to be unpinned — they don't ship in the image.
    let uninstalled = |pkg: &str| -> bool {
        for line in t.lines() {
            let l = line.trim();
            if !l.contains("pip uninstall") && !l.contains("pip3 uninstall") {
                continue;
            }
            if l.contains(pkg) {
                return true;
            }
        }
        false
    };

    for line in t.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("pip install") && !line.contains("pip3 install") {
            continue;
        }
        if line.contains(" -r ") {
            continue;
        }
        if line.contains("pip uninstall") {
            continue;
        }
        let after_install = match line.find("pip install") {
            Some(i) => &line[i + "pip install".len()..],
            None => match line.find("pip3 install") {
                Some(i) => &line[i + "pip3 install".len()..],
                None => continue,
            },
        };
        for tok in after_install.split_whitespace() {
            if tok.starts_with('-') || tok.starts_with('/') || tok.starts_with('$') {
                continue;
            }
            if tok == "\\" || tok == "&&" || tok == "||" || tok == ";" {
                break;
            }
            if tok.ends_with("uninstall") || tok.contains("&&") {
                break;
            }
            if tok.contains("==") || tok.contains(">=") || tok.contains("~=") {
                continue;
            }
            if tok.contains("git+") && (tok.contains("@") || tok.contains("#")) {
                continue;
            }
            if tok.ends_with(".tgz") || tok.ends_with(".whl") || tok.ends_with(".tar.gz") {
                continue;
            }
            if tok
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                && tok.len() > 1
            {
                // Transient build-time tools are exempt
                if uninstalled(tok) {
                    continue;
                }
                return true;
            }
        }
    }
    false
}

fn has_unpinned_npm(t: &str) -> bool {
    for line in t.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("npm install -g") && !line.contains("npm i -g") {
            continue;
        }
        let after = match line.find("-g") {
            Some(i) => &line[i + 2..],
            None => continue,
        };
        for tok in after.split_whitespace() {
            if tok.starts_with('-') || tok.starts_with('/') || tok.starts_with('$') {
                continue;
            }
            if tok == "\\" || tok == "&&" || tok == "||" || tok == ";" {
                break;
            }
            if tok.ends_with(".tgz") || tok.ends_with(".tar.gz") {
                continue;
            }
            // Pinned: contains `@` after the package portion
            // e.g. `@anthropic-ai/claude-code@2.1.104`
            // or   `openclaw@2026.4.11`
            // Strip leading `@` (scoped package), then look for another `@`
            let stripped = tok.strip_prefix('@').unwrap_or(tok);
            if stripped.contains('@') {
                continue;
            }
            if tok.len() > 1 {
                return true;
            }
        }
    }
    false
}

fn has_untagged_from(t: &str) -> bool {
    for line in t.lines() {
        let line = line.trim_start();
        // Dockerfile directive convention is UPPERCASE. Lowercase `from`
        // inside Python heredocs (`from huggingface_hub import ...`) would
        // otherwise false-positive.
        if !line.starts_with("FROM ") {
            continue;
        }
        let rest = line[5..].trim();
        if rest.is_empty() || rest.starts_with("scratch") || rest.starts_with('$') {
            continue;
        }
        let image = rest.split_whitespace().next().unwrap_or("");
        let last_slash = image.rfind('/').map(|i| i + 1).unwrap_or(0);
        let tail = &image[last_slash..];
        if !tail.contains(':') && !tail.contains('@') {
            return true;
        }
    }
    false
}

fn has_legacy_env_var(t: &str) -> bool {
    // Find references to unprefixed $TASK_ID / $BENCHMARK. Must NOT
    // match $DOCK_TASK_ID / $DOCK_BENCHMARK (prefixed), and must not
    // match longer identifiers like $TASK_ID_STR (substring false
    // positive). Whole-identifier match: the character after the
    // needle must not be an identifier continuation character.
    let ident_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    for needle in ["$TASK_ID", "${TASK_ID", "$BENCHMARK", "${BENCHMARK"] {
        let mut rest = t;
        while let Some(i) = rest.find(needle) {
            let before = &rest[..i];
            // Skip if preceded by DOCK_ (e.g. $DOCK_TASK_ID)
            let prefixed = before.ends_with("DOCK_");
            // Skip if followed by an identifier char (e.g. $TASK_ID_STR)
            let after = &rest[i + needle.len()..];
            let extended = after.chars().next().map(ident_char).unwrap_or(false);
            if !prefixed && !extended {
                return true;
            }
            rest = &rest[i + 1..];
        }
    }
    false
}

fn has_todo_or_fixme(t: &str) -> bool {
    for line in t.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        // Allow a documented FUTURE: block
        if trimmed.contains("FUTURE:") {
            continue;
        }
        for tok in ["TODO", "FIXME", "XXX"] {
            // standalone token check
            if trimmed
                .split(|c: char| !c.is_alphanumeric())
                .any(|w| w == tok)
            {
                return true;
            }
        }
    }
    false
}

fn strip_heredocs(t: &str) -> String {
    // Dockerfiles often write install scripts via `cat > file <<'NAME'`
    // heredocs. The body of those heredocs is not a RUN command — it's
    // content being written to a file. Installation rules (apt cleanup,
    // pip pinning, etc.) should skip heredoc bodies.
    let mut out = String::with_capacity(t.len());
    let mut in_heredoc: Option<String> = None;
    for line in t.lines() {
        if let Some(tag) = &in_heredoc {
            if line.trim() == tag {
                in_heredoc = None;
            }
            out.push('\n');
            continue;
        }
        // Detect `<<'TAG'` or `<<TAG` or `<<"TAG"`
        if let Some(i) = line.find("<<") {
            let after = &line[i + 2..];
            let after = after.trim_start_matches('-');
            let tag: String = after
                .trim_start_matches('\'')
                .trim_start_matches('"')
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !tag.is_empty() {
                in_heredoc = Some(tag);
                out.push_str(line);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn apt_install_without_cleanup(t: &str) -> bool {
    // Skip heredoc bodies, then join multi-line RUN continuations.
    let stripped = strip_heredocs(t);
    let joined = stripped.replace("\\\r\n", " ").replace("\\\n", " ");
    for line in joined.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("apt-get install") {
            continue;
        }
        if !line.contains("rm -rf /var/lib/apt/lists") {
            return true;
        }
    }
    false
}

fn pip_install_without_no_cache(t: &str) -> bool {
    let stripped = strip_heredocs(t);
    let joined = stripped.replace("\\\r\n", " ").replace("\\\n", " ");
    for line in joined.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("pip install") && !line.contains("pip3 install") {
            continue;
        }
        if line.contains("pip uninstall") {
            continue;
        }
        if !line.contains("--no-cache-dir") {
            return true;
        }
    }
    false
}

fn label_name_matches_dir(t: &str, dir: &str) -> bool {
    // Look for dock.benchmark.name or dock.agent.name and compare to dir.
    for line in t.lines() {
        let l = line.trim();
        if !l.starts_with("LABEL ") {
            continue;
        }
        for key in ["dock.benchmark.name=", "dock.agent.name="] {
            if let Some(i) = l.find(key) {
                let rest = &l[i + key.len()..];
                // Extract value between quotes if quoted
                let val = rest
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .split(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                return val == dir;
            }
        }
    }
    // No name label found — caller treats this as a separate check
    true
}

fn missing_dock_type(t: &str) -> bool {
    !t.contains(r#"LABEL dock.type="#)
}

// ─── New rules from the 2026-04-15 dockerfile audit walk ───────────

fn todo_string_literal(t: &str) -> bool {
    // Flag literal "TODO" or 'TODO' inside RUN steps — cybench /
    // mle-bench / swe-lancer write the word "TODO" as actual task
    // content when upstream JSON lacks required fields, so the image
    // then grades the agent against placeholder text. The existing
    // todo_or_fixme rule only looks at `#` comments and misses this.
    // Skip lines that are pure comments.
    for line in t.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        // Must be inside a RUN (best-effort: only look at lines
        // containing `"TODO"` or `'TODO'` as a quoted literal) to
        // avoid matching the word `TODO` in unrelated positions.
        if trimmed.contains("\"TODO\"") || trimmed.contains("'TODO'") {
            return true;
        }
    }
    false
}

fn silent_pip_fallback(t: &str) -> bool {
    // `pip install ... 2>/dev/null || pip3 install ...` swallows
    // stderr and falls back silently. If both fail, the dependency
    // is missing at runtime and grade.py raises ImportError which
    // test.sh converts to reward=0 — silent false negative.
    for line in t.lines() {
        let lc = line.to_lowercase();
        if (lc.contains("pip install") || lc.contains("pip3 install")) && lc.contains("2>/dev/null")
        {
            return true;
        }
        if (lc.contains("pip install") || lc.contains("pip3 install")) && lc.contains("|| true") {
            return true;
        }
    }
    false
}

fn install_order_pip_before_apt(t: &str) -> bool {
    // Layer-cache smell: pip install runs BEFORE apt-get install in
    // the file. pip layers are more volatile (frequent upgrades),
    // apt-get install is usually stable — running apt first lets the
    // stable layer cache while pip churns.
    let mut saw_pip_first = None;
    for (i, line) in t.lines().enumerate() {
        let lc = line.trim().to_lowercase();
        if lc.starts_with('#') {
            continue;
        }
        // Only consider top-level RUN lines
        if !lc.starts_with("run ") {
            continue;
        }
        if lc.contains("pip install") || lc.contains("pip3 install") {
            saw_pip_first.get_or_insert(i);
        } else if lc.contains("apt-get install")
            && let Some(pip_idx) = saw_pip_first
            && pip_idx < i
        {
            return true;
        }
    }
    false
}

fn phantom_pip_uninstall_in_separate_run(t: &str) -> bool {
    // `pip uninstall pyarrow` in its own RUN layer after a prior RUN
    // that installed it reclaims zero space — the install layer still
    // holds the files. The uninstall must be in the same RUN as the
    // install to actually shrink the image. Heuristic: a RUN line that
    // contains `pip uninstall` and no matching `pip install` on the
    // same line.
    for line in t.lines() {
        let lc = line.trim().to_lowercase();
        if lc.starts_with('#') {
            continue;
        }
        if !lc.starts_with("run ") {
            continue;
        }
        if lc.contains("pip uninstall") && !lc.contains("pip install") {
            return true;
        }
    }
    false
}

fn missing_data_revision_when_fetching_mutable_ref(t: &str) -> bool {
    // If a RUN step pulls from a mutable HuggingFace/GitHub ref
    // (refs/convert/parquet, main, master, HEAD) AND the image lacks
    // a dock.benchmark.data_revision label, upstream can silently
    // change the dataset under us.
    //
    // Only inspect RUN steps — LABEL values can contain "/main/"
    // innocuously (doc links, inspect_impl URLs). We can't just grep
    // the whole file.
    let joined = strip_heredocs(t)
        .replace("\\\r\n", " ")
        .replace("\\\n", " ");
    let mut has_mutable_fetch = false;
    for line in joined.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.to_lowercase().starts_with("run ") {
            continue;
        }
        if trimmed.contains("refs/convert/parquet")
            || trimmed.contains("?revision=main")
            || trimmed.contains("?revision=master")
            // github raw URL with a branch name in the path
            || trimmed.contains("raw.githubusercontent.com/") && (trimmed.contains("/main/") || trimmed.contains("/master/"))
        {
            has_mutable_fetch = true;
            break;
        }
    }
    if !has_mutable_fetch {
        return false;
    }
    // Allow if there's a data_revision label with a non-mutable value
    for line in t.lines() {
        if line.contains("dock.benchmark.data_revision=") {
            let rest = &line[line.find('=').unwrap() + 1..];
            let val = rest.trim_matches(['"', '\'', ' ', '\t']);
            if !val.is_empty()
                && val != "latest"
                && val != "main"
                && val != "master"
                && val != "HEAD"
            {
                return false;
            }
        }
    }
    true
}

fn upstream_base_unpinned(t: &str) -> bool {
    // Yellow signal: `dock.benchmark.upstream_base` label pins to :latest
    // (or leaves the tag unset, which is equivalent). Per benchmarks/RULES.md
    // principle 21b, third-party bases are legal but MUST be flagged as
    // supply-chain debt until mirrored or pinned by digest.
    for line in t.lines() {
        if let Some(i) = line.find("dock.benchmark.upstream_base=") {
            let rest = &line[i + "dock.benchmark.upstream_base=".len()..];
            let val = rest
                .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                .split(['"', '\''])
                .next()
                .unwrap_or("");
            // Strip any variable substitution before tag analysis.
            if val.ends_with(":latest") || (!val.contains(':') && !val.contains('@')) {
                return true;
            }
        }
    }
    false
}

fn data_revision_is_stale_pointer(t: &str) -> bool {
    // dock.benchmark.data_revision="latest|main|master|HEAD|''"
    for line in t.lines() {
        if let Some(i) = line.find("dock.benchmark.data_revision=") {
            let rest = &line[i + "dock.benchmark.data_revision=".len()..];
            let val = rest
                .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                .split(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                .next()
                .unwrap_or("");
            return val.is_empty()
                || val == "latest"
                || val == "main"
                || val == "master"
                || val == "HEAD";
        }
    }
    false
}

fn uses_full_python_when_slim_exists(t: &str) -> bool {
    // `FROM python:3.X` without -slim suffix (and not pointing at a
    // known-needs-headers variant like `-dev`). Uppercase-only match
    // to avoid catching Python heredoc `from ... import ...` lines.
    for line in t.lines() {
        let l = line.trim_start();
        if !l.starts_with("FROM ") {
            continue;
        }
        if let Some(rest) = l.get(5..) {
            let image = rest.split_whitespace().next().unwrap_or("");
            if image.starts_with("python:")
                && !image.contains("-slim")
                && !image.contains("-alpine")
                && !image.contains("-dev")
            {
                return true;
            }
        }
    }
    false
}

// ─── Type predicates (RULES.md principle 9: version-override axis) ─

fn is_benchmark(t: &str) -> bool {
    t.contains(r#"LABEL dock.type="benchmark""#)
}
fn is_agent(t: &str) -> bool {
    t.contains(r#"LABEL dock.type="agent""#)
}
fn is_model(t: &str) -> bool {
    t.contains(r#"LABEL dock.type="model""#)
}

// Exemption: models/replay is the in-repo replay stub that does NOT
// wrap the real LiteLLM proxy — it implements its own minimal HTTP
// server. It has no litellm_version to record.
fn is_replay_model(_t: &str, dir: &str) -> bool {
    dir == "replay"
}

fn benchmark_missing_version_default(t: &str, _dir: &str) -> bool {
    is_benchmark(t) && !t.contains("ENV DOCK_BENCHMARK_VERSION_DEFAULT=")
}
fn agent_missing_version_default(t: &str, _dir: &str) -> bool {
    is_agent(t) && !t.contains("ENV DOCK_AGENT_VERSION_DEFAULT=")
}
fn model_missing_litellm_version_label(t: &str, dir: &str) -> bool {
    is_model(t) && !is_replay_model(t, dir) && !t.contains("LABEL dock.model.litellm_version=")
}
fn model_missing_litellm_version_default(t: &str, dir: &str) -> bool {
    is_model(t) && !is_replay_model(t, dir) && !t.contains("ENV DOCK_LITELLM_VERSION_DEFAULT=")
}

// ─── Rule catalog (data, not code) ─────────────────────────────────

const RULES: &[Rule] = &[
    // ── Red ─────────────────────────────────────────────────────────
    Rule::red(
        "missing_dock_type",
        "Dockerfile is missing a LABEL dock.type= declaration",
        |t, _| missing_dock_type(t),
    ),
    Rule::red(
        "hardcoded_secret",
        "Dockerfile contains a literal API key or credential",
        |t, _| contains_hardcoded_api_key(t),
    ),
    Rule::red(
        "untagged_from",
        "FROM without a tag — image is not reproducible",
        |t, _| has_untagged_from(t),
    ),
    Rule::red(
        "legacy_env_var",
        "references $TASK_ID or $BENCHMARK — must use $DOCK_TASK_ID / $DOCK_BENCHMARK",
        |t, _| has_legacy_env_var(t),
    ),
    Rule::red(
        "label_dir_mismatch",
        "dock.benchmark.name / dock.agent.name label does not match directory name",
        |t, dir| !label_name_matches_dir(t, dir),
    ),
    Rule::red(
        "apt_no_cleanup",
        "apt-get install without rm -rf /var/lib/apt/lists/* on the same RUN (RULES.md 10b)",
        |t, _| apt_install_without_cleanup(t),
    ),
    Rule::red(
        "pip_no_cache_flag",
        "pip install without --no-cache-dir (RULES.md 10b)",
        |t, _| pip_install_without_no_cache(t),
    ),
    Rule::red(
        "unpinned_pip",
        "pip install without ==version pin",
        |t, _| has_unpinned_pip(t),
    ),
    Rule::red(
        "unpinned_npm",
        "npm install -g without @version pin",
        |t, _| has_unpinned_npm(t),
    ),
    Rule::red(
        "todo_or_fixme",
        "Dockerfile comment contains TODO/FIXME/XXX (use FUTURE: for explicit future work)",
        |t, _| has_todo_or_fixme(t),
    ),
    // ── Yellow ──────────────────────────────────────────────────────
    Rule::yellow(
        "stale_data_revision",
        "dock.benchmark.data_revision is empty, latest, main, master, or HEAD",
        |t, _| data_revision_is_stale_pointer(t),
    ),
    Rule::yellow(
        "python_full_base",
        "FROM python:X without -slim suffix (RULES.md 10a)",
        |t, _| uses_full_python_when_slim_exists(t),
    ),
    Rule::yellow(
        "upstream_base_unpinned",
        "dock.benchmark.upstream_base pins :latest — third-party registry, supply-chain debt (benchmarks/RULES.md 21b)",
        |t, _| upstream_base_unpinned(t),
    ),
    // ── New rules from the 2026-04-15 dockerfile audit walk ────────
    Rule::red(
        "todo_string_literal",
        "Dockerfile writes the literal string \"TODO\" as task data (silent placeholder grading)",
        |t, _| todo_string_literal(t),
    ),
    Rule::red(
        "silent_pip_fallback",
        "pip install with 2>/dev/null or || true fallback — errors are swallowed, grade.py will silently fail",
        |t, _| silent_pip_fallback(t),
    ),
    Rule::yellow(
        "install_order_pip_before_apt",
        "pip install runs before apt-get install — reverse the order so the stable apt layer can cache",
        |t, _| install_order_pip_before_apt(t),
    ),
    Rule::yellow(
        "phantom_pip_uninstall",
        "pip uninstall in its own RUN layer reclaims no space (RULES.md 10b) — combine with the install",
        |t, _| phantom_pip_uninstall_in_separate_run(t),
    ),
    Rule::yellow(
        "missing_data_revision_when_fetching_mutable_ref",
        "Dockerfile fetches from a mutable ref (refs/convert/parquet, main, master) without pinning dock.benchmark.data_revision",
        |t, _| missing_data_revision_when_fetching_mutable_ref(t),
    ),
    // ── Version-override contract (RULES.md principle 9) ──────────
    // Each image declares its baked-in upstream version as an ENV so
    // core/entrypoint/dock-entrypoint.sh can compare it to a runtime
    // DOCK_*_VERSION override and decide whether to refetch/reinstall.
    Rule::red(
        "benchmark_missing_version_default",
        "benchmark Dockerfile is missing ENV DOCK_BENCHMARK_VERSION_DEFAULT (RULES.md 9)",
        benchmark_missing_version_default,
    ),
    Rule::red(
        "agent_missing_version_default",
        "agent Dockerfile is missing ENV DOCK_AGENT_VERSION_DEFAULT (RULES.md 9)",
        agent_missing_version_default,
    ),
    Rule::red(
        "model_missing_litellm_version_label",
        "model Dockerfile is missing LABEL dock.model.litellm_version (models/RULES.md 15)",
        model_missing_litellm_version_label,
    ),
    Rule::red(
        "model_missing_litellm_version_default",
        "model Dockerfile is missing ENV DOCK_LITELLM_VERSION_DEFAULT (RULES.md 9)",
        model_missing_litellm_version_default,
    ),
];

// ─── Engine ────────────────────────────────────────────────────────

#[derive(Debug)]
struct Finding {
    path: PathBuf,
    rule: &'static str,
    severity: Severity,
    why: &'static str,
}

fn inspect_dockerfile(path: &Path, text: &str, dir: &str) -> Vec<Finding> {
    RULES
        .iter()
        .filter(|r| (r.test)(text, dir))
        .map(|r| Finding {
            path: path.to_path_buf(),
            rule: r.id,
            severity: r.severity,
            why: r.why,
        })
        .collect()
}

// ─── Discovery ─────────────────────────────────────────────────────

fn walk_dockerfiles() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for root in ["benchmarks", "agents", "models"] {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            let dockerfile = path.join("Dockerfile");
            if dockerfile.is_file() {
                out.push((dockerfile, dir));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// ─── Unit tests (always run, no --ignored) ─────────────────────────

#[test]
fn rule_missing_dock_type_fires() {
    let bad = "FROM alpine:3\nRUN echo hi\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "missing_dock_type"));
}

#[test]
fn rule_hardcoded_secret_fires() {
    let bad = "FROM alpine:3\nENV OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz0123456789abcdefghij\nLABEL dock.type=\"agent\"\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "hardcoded_secret"));
}

#[test]
fn rule_untagged_from_fires() {
    let bad = "FROM ubuntu\nLABEL dock.type=\"agent\"\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "untagged_from"));
}

#[test]
fn rule_untagged_from_allows_scratch() {
    let ok = "FROM scratch\nLABEL dock.type=\"agent\"\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "untagged_from"));
}

#[test]
fn rule_legacy_env_var_fires() {
    let bad = "FROM alpine:3\nLABEL dock.type=\"benchmark\"\nRUN echo $TASK_ID\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "legacy_env_var"));
}

#[test]
fn rule_legacy_env_var_allows_dock_prefix() {
    let ok = "FROM alpine:3\nLABEL dock.type=\"benchmark\"\nRUN echo $DOCK_TASK_ID\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "legacy_env_var"));
}

#[test]
fn rule_label_dir_mismatch_fires() {
    let bad = "FROM alpine:3\nLABEL dock.type=\"benchmark\"\nLABEL dock.benchmark.name=\"other\"\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "mybench");
    assert!(fs.iter().any(|f| f.rule == "label_dir_mismatch"));
}

#[test]
fn rule_apt_cleanup_fires() {
    let bad = "FROM ubuntu:24.04\nLABEL dock.type=\"agent\"\nRUN apt-get update && apt-get install -y curl\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "apt_no_cleanup"));
}

#[test]
fn rule_apt_cleanup_allows_inline_rm() {
    let ok = "FROM ubuntu:24.04\nLABEL dock.type=\"agent\"\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "apt_no_cleanup"));
}

#[test]
fn rule_todo_or_fixme_fires() {
    let bad = "FROM alpine:3\nLABEL dock.type=\"agent\"\n# TODO: fix this\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "todo_or_fixme"));
}

#[test]
fn rule_todo_allows_future_block() {
    let ok = "FROM alpine:3\nLABEL dock.type=\"agent\"\n# FUTURE: consider swapping to alpine\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "todo_or_fixme"));
}

// ─── Fleet sweep (always runs — it's pure file I/O, <100ms) ────────

#[test]
fn inspect_every_dockerfile() {
    let dockerfiles = walk_dockerfiles();
    assert!(
        !dockerfiles.is_empty(),
        "no Dockerfiles found under benchmarks/ or agents/"
    );

    let mut all: Vec<Finding> = Vec::new();
    let mut read_errors: Vec<String> = Vec::new();

    for (path, dir) in &dockerfiles {
        match fs::read_to_string(path) {
            Ok(text) => all.extend(inspect_dockerfile(path, &text, dir)),
            Err(e) => read_errors.push(format!("{}: {e}", path.display())),
        }
    }

    let red: Vec<&Finding> = all.iter().filter(|f| f.severity == Severity::Red).collect();
    let yellow: Vec<&Finding> = all
        .iter()
        .filter(|f| f.severity == Severity::Yellow)
        .collect();

    eprintln!(
        "\n─── dockerfile inspection over {} files ───",
        dockerfiles.len()
    );
    if !yellow.is_empty() {
        eprintln!("\n{} yellow findings:", yellow.len());
        for f in &yellow {
            eprintln!("  {} ({}): {}", f.path.display(), f.rule, f.why);
        }
    }
    if !read_errors.is_empty() {
        eprintln!("\n{} read errors:", read_errors.len());
        for e in &read_errors {
            eprintln!("  {e}");
        }
    }
    if red.is_empty() && read_errors.is_empty() {
        eprintln!(
            "\n✓ all {} Dockerfiles healthy ({} yellow warnings)",
            dockerfiles.len(),
            yellow.len()
        );
        return;
    }

    let mut msg = String::new();
    if !red.is_empty() {
        msg.push_str(&format!("\n{} red findings:\n", red.len()));
        for f in &red {
            msg.push_str(&format!("  {} ({}): {}\n", f.path.display(), f.rule, f.why));
        }
    }
    if !read_errors.is_empty() {
        msg.push_str(&format!("\n{} read errors:\n", read_errors.len()));
        for e in &read_errors {
            msg.push_str(&format!("  {e}\n"));
        }
    }
    panic!("{msg}");
}
