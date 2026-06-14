//! Shared helpers for the fleet test crates — `tests/static`, `tests/build`,
//! and `tests/run`.
//!
//! The fleet's files are addressed repo-root-relative (`containers/...`,
//! `deploy/...`, `tests/...`), but `cargo test` runs each target with its own
//! crate directory as the working directory. These helpers locate the repo root
//! independent of which test crate is calling, so the same path logic works
//! from every stage.

use std::path::{Path, PathBuf};
use std::sync::Once;

/// Absolute path to the repository root (the workspace root).
///
/// Found by walking up from this crate's compile-time manifest directory until a
/// directory holds both the `containers/` fleet and the `cli/` crate's
/// `Cargo.toml`. The `cli/Cargo.toml` *file* (not just a `cli/` dir) is the
/// discriminator: the doc-only `tests/cli/` directory would otherwise make the
/// `tests/` dir look like the root. Anchoring to `CARGO_MANIFEST_DIR` (this
/// crate is always at `tests/support`) rather than the process working
/// directory makes it robust both to [`enter_repo_root`] having already changed
/// the cwd and to the test crates living one level deeper under `tests/`.
pub fn repo_root() -> PathBuf {
    let start = Path::new(env!("CARGO_MANIFEST_DIR"));
    start
        .ancestors()
        .find(|dir| dir.join("containers").is_dir() && dir.join("cli/Cargo.toml").is_file())
        .unwrap_or_else(|| {
            panic!(
                "repo root not found above {} (looked for a dir containing containers/ and cli/Cargo.toml)",
                start.display()
            )
        })
        .to_path_buf()
}

static ENTER: Once = Once::new();

/// Set the process working directory to the repository root, once per test
/// binary. Tests that shell out to `docker`/`helm` (which inherit cwd) or call
/// `eval_containers::bake` discovery (which reads `containers/<category>`
/// relative to cwd) call this first. `Once` makes it safe under the test
/// harness's parallel threads: every caller blocks until the first has set the
/// directory, so no test observes the pre-`cd` cwd.
pub fn enter_repo_root() {
    ENTER.call_once(|| {
        std::env::set_current_dir(repo_root()).expect("set current dir to repo root");
    });
}
