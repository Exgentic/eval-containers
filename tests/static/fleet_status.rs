//! Freshness comparison — the rule-14 judgment (delivery/RULES.md): recorded
//! build-input hash vs the repository's computed hash, absent/unreadable
//! failing dirty.
//!
//! `containers/scripts/fleet-status.sh` reads registry labels via `imagetools
//! inspect`, so the offline test (tests/static/RULES.md rule 1) stubs `docker`
//! on PATH with canned responses covering every read shape: a multi-arch
//! manifest list carrying attestation entries, a single-arch config object, a
//! labeled-but-hashless image, and an absent ref. The ref derivation (graph
//! context column, dot-safe for models like gpt-5.4) is asserted against the
//! real repo.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use test_support::repo_root;

/// PATH-shimmed fake `docker`, removed on drop.
struct Stub(PathBuf);

impl Drop for Stub {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_stub(script_body: &str) -> Stub {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("fleet-status-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("docker");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    Stub(dir)
}

/// ref -> (verdict, computed, recorded)
fn fleet_status(stub: &Stub) -> HashMap<String, (String, String, String)> {
    let root = repo_root();
    let path = format!(
        "{}:{}",
        stub.0.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("bash")
        .arg(root.join("containers/scripts/fleet-status.sh"))
        .env("PATH", path)
        .env("STATUS_JOBS", "8")
        .output()
        .expect("run fleet-status.sh");
    assert!(
        out.status.success(),
        "fleet-status failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 4, "malformed row: {l}");
            (
                f[0].to_string(),
                (f[1].to_string(), f[2].to_string(), f[3].to_string()),
            )
        })
        .collect()
}

/// Every verdict class, every registry read shape, and the dot-safe ref map,
/// on the real repo with a stubbed registry.
#[test]
fn verdicts_cover_every_read_shape() {
    let root = repo_root();
    let hashes = Command::new("bash")
        .arg(root.join("containers/scripts/fleet-hash.sh"))
        .output()
        .expect("run fleet-hash.sh");
    assert!(hashes.status.success());
    let aime = String::from_utf8(hashes.stdout)
        .unwrap()
        .lines()
        .find(|l| l.starts_with("benchmark-aime\t"))
        .expect("aime row")
        .split('\t')
        .nth(1)
        .unwrap()
        .to_string();

    // aime: fresh via a manifest list (attestation entry must be ignored);
    // gsm8k: stale via a single-arch config object; arc: labels but no hash.
    // Everything else: inspect fails => absent.
    let stub = write_stub(&format!(
        r#"ref="$4"
case "$ref" in
  */benchmarks/aime:latest)
    echo '{{"linux/amd64":{{"config":{{"Labels":{{"eval.input-hash":"{aime}"}}}}}},"unknown/unknown":{{"config":{{}}}}}}' ;;
  */benchmarks/gsm8k:latest)
    echo '{{"config":{{"Labels":{{"eval.input-hash":"deadbeef"}}}}}}' ;;
  */benchmarks/arc:latest)
    echo '{{"linux/amd64":{{"config":{{"Labels":{{"other":"x"}}}}}}}}' ;;
  *) exit 1 ;;
esac
"#
    ));
    let rows = fleet_status(&stub);
    assert_eq!(rows.len(), 153, "one row per static bake target");

    let (v, want, got) = &rows["ghcr.io/exgentic/benchmarks/aime:latest"];
    assert_eq!((v.as_str(), got), ("fresh", want));
    assert_eq!(want, &aime);
    assert_eq!(rows["ghcr.io/exgentic/benchmarks/gsm8k:latest"].0, "stale");
    assert_eq!(
        rows["ghcr.io/exgentic/benchmarks/gsm8k:latest"].2,
        "deadbeef"
    );
    assert_eq!(
        rows["ghcr.io/exgentic/benchmarks/arc:latest"].0,
        "unlabeled"
    );
    assert_eq!(rows["ghcr.io/exgentic/core/entrypoint:latest"].0, "absent");

    // The ref map preserves dots that bake target names cannot carry.
    assert!(rows.contains_key("ghcr.io/exgentic/models/gpt-5.4:latest"));
    assert!(rows.contains_key("ghcr.io/exgentic/models/gpt-4.1-mini:latest"));

    // Rule 14: everything non-fresh is "changed"; exactly one image was fresh.
    let fresh = rows.values().filter(|r| r.0 == "fresh").count();
    assert_eq!(fresh, 1);
}
