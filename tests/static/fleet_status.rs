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

/// The one-ref `check` form: (ref, verdict, computed, recorded, platforms).
fn check_one(stub: &Stub, image: &str, hash: &str) -> (String, String, String, String, String) {
    let out = Command::new("bash")
        .arg(repo_root().join("containers/scripts/fleet-status.sh"))
        .args(["check", image, hash])
        .env(
            "PATH",
            format!(
                "{}:{}",
                stub.0.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("STATUS_RETRIES", "3")
        .output()
        .expect("run fleet-status.sh check");
    let line = String::from_utf8(out.stdout).expect("utf8");
    let f: Vec<&str> = line.trim_end().split('\t').collect();
    assert_eq!(f.len(), 5, "malformed row: {line}");
    (
        f[0].into(),
        f[1].into(),
        f[2].into(),
        f[3].into(),
        f[4].into(),
    )
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
    assert_eq!(rows.len(), 159, "one row per static bake target");

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

/// A dropped connection is not a missing image. ghcr fails often enough that
/// treating any failed read as `absent` made two sweeps minutes apart disagree
/// by six images — and each false absent is a needless rebuild under rule 14.
/// Transient errors retry; a registry that never answers reads `unreadable`
/// (still changed, but distinguishable); a real not-found stays `absent` and
/// does not burn retries.
#[test]
fn transient_registry_errors_retry_instead_of_reading_absent() {
    let hash = input_hash("benchmark-aime");
    let counter = std::env::temp_dir().join(format!("fs-attempts-{}", std::process::id()));
    let _ = std::fs::remove_file(&counter);

    let flaky = write_stub(&format!(
        r#"n=$(cat {c} 2>/dev/null || echo 0); n=$((n+1)); echo $n > {c}
if [ "$n" -lt 3 ]; then echo 'ERROR: failed to do request: dial tcp 20.0.0.1:443: i/o timeout' >&2; exit 1; fi
echo '{{"image":{{"linux/amd64":{{"config":{{"Labels":{{"eval.input-hash":"{hash}"}}}}}}}},"manifest":{{"manifests":[{{"platform":{{"os":"linux","architecture":"amd64"}}}}]}}}}'
"#,
        c = counter.display(),
    ));
    let out = check_one(&flaky, "ghcr.io/exgentic/benchmarks/aime:latest", &hash);
    assert_eq!(out.1, "fresh", "a blip must not read as absent: {out:?}");
    assert_eq!(
        std::fs::read_to_string(&counter).unwrap().trim(),
        "3",
        "it must actually have retried"
    );
    let _ = std::fs::remove_file(&counter);

    let dead =
        write_stub("echo 'ERROR: failed to do request: dial tcp: i/o timeout' >&2; exit 1\n");
    assert_eq!(
        check_one(&dead, "ghcr.io/exgentic/benchmarks/aime:latest", &hash).1,
        "unreadable",
        "a registry that never answers is unreadable, not absent"
    );

    let missing = write_stub("echo 'ERROR: ghcr.io/x/y:latest: not found' >&2; exit 1\n");
    assert_eq!(
        check_one(&missing, "ghcr.io/exgentic/benchmarks/aime:latest", &hash).1,
        "absent",
        "a genuine not-found is still absent"
    );
}

/// ref -> (verdict, computed, recorded) for the `compose` form.
fn compose_status(stub: &Stub) -> HashMap<String, (String, String, String)> {
    let out = Command::new("bash")
        .arg(repo_root().join("containers/scripts/fleet-status.sh"))
        .args(["compose", "latest"])
        .env(
            "PATH",
            format!(
                "{}:{}",
                stub.0.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("REGISTRY", "ghcr.io/exgentic")
        .output()
        .expect("run fleet-status.sh compose");
    assert!(
        out.status.success(),
        "compose sweep failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 4, "malformed row: {l}");
            (f[0].into(), (f[1].into(), f[2].into(), f[3].into()))
        })
        .collect()
}

/// An `eval-<benchmark>` artifact is the flattened compose bytes verbatim, so
/// its published layer digest IS its content hash — that is what makes the
/// shared `containers/compose/` half an input, though it sits in no image's
/// build context (#451). Same fail-dirty verdicts as an image, plus `declined`
/// for a stack that opts out of publishing.
#[test]
fn compose_artifacts_compare_the_flatten_against_the_published_layer() {
    // sha256 of the stub's `docker compose config` output ("FLAT\n").
    const FLAT: &str = "e5c6520d56a2b2b69b9bafeae345e01527ebdf79cfbfadb0a9718ba16a8b052f";
    let stub = write_stub(&format!(
        r#"if [ "$1" = compose ]; then                 # docker compose -f <file> config …
  case "$3" in *arc-agi/*) exit 1 ;; esac    # a stack that will not flatten
  printf 'FLAT\n'; exit 0
fi
case "$4" in                       # docker buildx imagetools inspect <ref> --raw
  *eval-aime:*) echo '{{"layers":[{{"digest":"sha256:{FLAT}"}}]}}' ;;
  *eval-mmmu:*) echo '{{"layers":[{{"digest":"sha256:beef"}}]}}' ;;
  *) echo 'ERROR: ghcr.io/x/y:latest: not found' >&2; exit 1 ;;
esac
"#
    ));
    let rows = compose_status(&stub);
    let verdict = |b: &str| {
        rows.get(&format!("ghcr.io/exgentic/eval-{b}:latest"))
            .unwrap_or_else(|| panic!("no row for eval-{b}"))
            .clone()
    };

    assert_eq!(
        verdict("aime"),
        ("fresh".into(), FLAT.into(), FLAT.into()),
        "published layer == the local flatten is the only way to read fresh"
    );
    assert_eq!(
        verdict("mmmu").0,
        "stale",
        "a different layer digest is stale"
    );
    assert_eq!(
        verdict("gsm8k").0,
        "absent",
        "an artifact that was never published fails dirty, not fresh"
    );
    assert_eq!(
        verdict("arc-agi").0,
        "unflattenable",
        "a stack whose flatten fails must not read fresh either"
    );
    // osworld / tau-bench declare `x-eval-publish: false` — they have no
    // artifact, so they must not be scheduled for publishing on every push.
    for b in ["osworld", "tau-bench"] {
        assert_eq!(
            verdict(b).0,
            "declined",
            "{b} declares x-eval-publish: false"
        );
    }
}
