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
//!
//! Platform completeness is covered too: a hash-matching image missing an
//! expected arch must read `partial`, not `fresh` — otherwise a one-arch build
//! failure is invisible forever (the surviving arch's label still matches) and
//! the missing arch never rebuilds. An image that *declares* a smaller platform
//! set via `LABEL eval.platforms` is held to that set instead.

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

/// ref -> (verdict, computed, recorded, platforms)
fn fleet_status(stub: &Stub) -> HashMap<String, (String, String, String, String)> {
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
            assert_eq!(f.len(), 5, "malformed row: {l}");
            (
                f[0].to_string(),
                (
                    f[1].to_string(),
                    f[2].to_string(),
                    f[3].to_string(),
                    f[4].to_string(),
                ),
            )
        })
        .collect()
}

fn input_hash(target: &str) -> String {
    let out = Command::new("bash")
        .arg(repo_root().join("containers/scripts/fleet-hash.sh"))
        .output()
        .expect("run fleet-hash.sh");
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .find(|l| l.starts_with(&format!("{target}\t")))
        .unwrap_or_else(|| panic!("{target} row"))
        .split('\t')
        .nth(1)
        .unwrap()
        .to_string()
}

/// An OCI index carrying `archs` plus the attestation entry that rides along at
/// unknown/unknown, with `hash` recorded on every arch's config.
fn index_json(archs: &[&str], hash: &str) -> String {
    let manifests: Vec<String> = archs
        .iter()
        .map(|a| format!(r#"{{"platform":{{"os":"linux","architecture":"{a}"}}}}"#))
        .chain(std::iter::once(
            r#"{"annotations":{"vnd.docker.reference.type":"attestation-manifest"},"platform":{"os":"unknown","architecture":"unknown"}}"#.to_string(),
        ))
        .collect();
    let image: Vec<String> = archs
        .iter()
        .map(|a| format!(r#""linux/{a}":{{"config":{{"Labels":{{"eval.input-hash":"{hash}"}}}}}}"#))
        .collect();
    format!(
        r#"{{"image":{{{},"unknown/unknown":{{"config":{{}}}}}},"manifest":{{"manifests":[{}]}}}}"#,
        image.join(","),
        manifests.join(",")
    )
}

/// Every verdict class, every registry read shape, and the dot-safe ref map,
/// on the real repo with a stubbed registry.
#[test]
fn verdicts_cover_every_read_shape() {
    let aime = input_hash("benchmark-aime");
    let mmmu = input_hash("benchmark-mmmu");
    let appworld = input_hash("benchmark-appworld");

    // aime: fresh — hash matches on both expected arches (attestation ignored).
    // mmmu: hash matches but arm64 never published => partial, not fresh.
    // appworld: amd64 only, and it declares eval.platforms="linux/amd64" => fresh.
    // gsm8k: stale via a plain single-arch manifest. arc: labels but no hash.
    // Everything else: inspect fails => absent.
    let stub = write_stub(&format!(
        r#"ref="$4"
case "$ref" in
  */benchmarks/aime:latest)     echo '{aime_json}' ;;
  */benchmarks/mmmu:latest)     echo '{mmmu_json}' ;;
  */benchmarks/appworld:latest) echo '{appworld_json}' ;;
  */benchmarks/gsm8k:latest)
    echo '{{"image":{{"os":"linux","architecture":"amd64","config":{{"Labels":{{"eval.input-hash":"deadbeef"}}}}}},"manifest":{{}}}}' ;;
  */benchmarks/arc:latest)      echo '{arc_json}' ;;
  *) exit 1 ;;
esac
"#,
        aime_json = index_json(&["amd64", "arm64"], &aime),
        mmmu_json = index_json(&["amd64"], &mmmu),
        appworld_json = index_json(&["amd64"], &appworld),
        arc_json = r#"{"image":{"linux/amd64":{"config":{"Labels":{"other":"x"}}}},"manifest":{"manifests":[{"platform":{"os":"linux","architecture":"amd64"}}]}}"#,
    ));
    let rows = fleet_status(&stub);
    assert_eq!(rows.len(), 153, "one row per static bake target");

    let (v, want, got, plats) = &rows["ghcr.io/exgentic/benchmarks/aime:latest"];
    assert_eq!((v.as_str(), got), ("fresh", want));
    assert_eq!(want, &aime);
    assert_eq!(plats, "linux/amd64,linux/arm64");

    // The arch-completeness gap: hash matches, yet the image is not fresh.
    let mmmu_row = &rows["ghcr.io/exgentic/benchmarks/mmmu:latest"];
    assert_eq!(
        mmmu_row.0, "partial",
        "amd64-only image must not read fresh"
    );
    assert_eq!(mmmu_row.2, mmmu, "the recorded hash still matches");
    assert!(
        mmmu_row.3.contains("missing:linux/arm64"),
        "row must name the missing arch, got {:?}",
        mmmu_row.3
    );

    // ...unless the image declares it is single-arch on purpose.
    assert_eq!(
        rows["ghcr.io/exgentic/benchmarks/appworld:latest"].0, "fresh",
        "eval.platforms=\"linux/amd64\" makes an amd64-only image complete"
    );

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

    // Rule 14: everything non-fresh is "changed" — aime and appworld are fresh,
    // and the partial one counts as changed alongside stale/unlabeled/absent.
    let fresh = rows.values().filter(|r| r.0 == "fresh").count();
    assert_eq!(fresh, 2);
    assert_eq!(rows.values().filter(|r| r.0 == "partial").count(), 1);
}
