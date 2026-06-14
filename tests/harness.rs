//! Library target for the `eval-containers-tests` package.
//!
//! This crate exists to host the fleet's integration-test suite — the
//! `[[test]]` targets declared in `Cargo.toml`, each a standalone
//! `<category>/test.rs` that drives the `eval_containers` CLI library against
//! the real container fleet in `../containers`. See `tests/RULES.md` for the
//! overall strategy. The only library API is the path helpers below, which
//! every test uses to locate fleet files independent of the working directory.

use std::path::{Path, PathBuf};
use std::sync::Once;

/// Absolute path to the repository root — the workspace root, i.e. the parent
/// of this `tests/` crate. Resolved from the compile-time manifest directory so
/// it is independent of the process working directory. The fleet's files are
/// addressed repo-root-relative (`containers/...`, `deploy/...`, `tests/...`),
/// but `cargo test` runs each target with the crate directory as cwd — so tests
/// either join this root or call [`enter_repo_root`].
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tests/ crate always has a parent (the repo root)")
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
