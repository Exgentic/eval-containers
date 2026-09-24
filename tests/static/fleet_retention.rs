//! Retention census — the rule-22 judgment (delivery/RULES.md): which published
//! digests nothing resolves to any more, and which only *look* that way.
//!
//! `containers/scripts/fleet-retention.py` reads two HTTP APIs — the GitHub
//! packages REST API for version ids and dates, the registry `/v2/` API to
//! resolve a manifest's children. PATH-shimming (the trick
//! `tests/static/fleet_status.rs` uses for `docker`) cannot reach an in-process
//! `urllib` call, so the script takes `RETENTION_API_BASE` /
//! `RETENTION_REGISTRY_BASE` and these tests serve canned JSON from a
//! `TcpListener` on `127.0.0.1`. A loopback fixture is not a network call under
//! tests/static/RULES.md rule 1, and it keeps the production path honest —
//! real pagination, real header parsing, which is where the bugs live.
//!
//! The headline assertion is `manifest_list_children_are_never_candidates`. An
//! untagged version is NOT a dangling one: a multi-arch image is an index whose
//! per-arch and attestation children are themselves versions carrying no tags,
//! so the arm64 half of the `latest` everyone pulls is indistinguishable from
//! garbage to a date-or-tag-only sweep. Measured on the real
//! `benchmarks/aime`: 3 of the 4 children of the live `:latest` are untagged.
//! Every other test here exists to pin one more way this tool could nominate a
//! live digest.
//!
//! Time is frozen through `RETENTION_NOW` so the 90-day boundary assertions are
//! not time bombs.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use test_support::repo_root;

/// "Now", for every fixture below. The cutoff is 90 days earlier: 2026-06-17.
const NOW: &str = "2026-09-15T00:00:00Z";
const ANCIENT: &str = "2024-01-01T00:00:00Z"; // far outside any window
const RECENT: &str = "2026-09-01T00:00:00Z"; // inside the 90-day window

/// A canned reply: an HTTP status and a body.
#[derive(Clone)]
struct Reply(u16, String);

/// A loopback HTTP server answering from a path -> reply table, counting hits.
///
/// Paths are matched by prefix on the request target, longest first, so a test
/// can pin `/v2/.../manifests/sha256:abc` without spelling every query string.
struct Server {
    base: String,
    hits: Arc<Mutex<HashMap<String, usize>>>,
    stop: Arc<AtomicUsize>,
}

impl Server {
    fn start(routes: Vec<(String, Vec<Reply>)>) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let base = format!("http://{}", listener.local_addr().unwrap());
        let hits: Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(HashMap::new()));
        let stop = Arc::new(AtomicUsize::new(0));
        let (h, s) = (hits.clone(), stop.clone());

        // Longest prefix first: a specific manifest route must win over `/v2/`.
        let mut routes = routes;
        routes.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));

        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if s.load(Ordering::SeqCst) == 1 {
                    return;
                }
                let Ok(mut conn) = conn else { continue };
                let mut reader = BufReader::new(conn.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line.is_empty() {
                    continue;
                }
                // "GET /path?query HTTP/1.1"
                let target = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                let mut header = String::new();
                while reader
                    .read_line(&mut header)
                    .map(|n| n > 2)
                    .unwrap_or(false)
                {
                    header.clear();
                }

                let matched = routes.iter().find(|(p, _)| target.starts_with(p.as_str()));
                let n = {
                    let mut hits = h.lock().unwrap();
                    let key = matched.map(|(p, _)| p.clone()).unwrap_or_default();
                    let e = hits.entry(key).or_insert(0);
                    *e += 1;
                    *e
                };
                let reply = match matched {
                    // The nth hit gets the nth reply; the last one repeats, so a
                    // flaky-then-healthy sequence is just a two-element list.
                    Some((_, replies)) => replies[(n - 1).min(replies.len() - 1)].clone(),
                    None => Reply(404, r#"{"message":"Not Found"}"#.into()),
                };
                // A generous rate-limit budget: throttling is not what these
                // tests pin, and a low remaining would make them sleep.
                let body = reply.1;
                let _ = write!(
                    conn,
                    "HTTP/1.1 {} X\r\nContent-Type: application/json\r\n\
                     X-RateLimit-Remaining: 4999\r\nX-RateLimit-Reset: 4000000000\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.0,
                    body.len(),
                    body
                );
            }
        });
        Server { base, hits, stop }
    }

    fn hits(&self, prefix: &str) -> usize {
        *self.hits.lock().unwrap().get(prefix).unwrap_or(&0)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(1, Ordering::SeqCst);
        // Unblock the accept loop so the thread can observe the stop flag.
        let _ = std::net::TcpStream::connect(self.base.trim_start_matches("http://"));
    }
}

/// One version row as the packages API returns it.
fn version(id: u64, digest: &str, created: &str, tags: &[&str]) -> String {
    let tags = tags
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"id":{id},"name":"sha256:{digest}","created_at":"{created}",
            "updated_at":"{created}","metadata":{{"container":{{"tags":[{tags}]}}}}}}"#
    )
}

/// An OCI index listing `children` as `(digest, os/arch, is-attestation)`.
fn index(children: &[(&str, &str, bool)]) -> String {
    let entries = children
        .iter()
        .map(|(d, plat, attest)| {
            let (os, arch) = plat.split_once('/').unwrap();
            let ann = if *attest {
                r#","annotations":{"vnd.docker.reference.type":"attestation-manifest"}"#
            } else {
                ""
            };
            format!(
                r#"{{"digest":"sha256:{d}","platform":{{"os":"{os}","architecture":"{arch}"}}{ann}}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{entries}]}}"#)
}

/// A one-package `/orgs/{org}/packages` listing.
fn listing(names: &[&str]) -> String {
    let rows = names
        .iter()
        .map(|n| format!(r#"{{"name":"{n}","repository":{{"name":"eval-containers"}}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rows}]")
}

/// Run the sweep against `server`, returning the parsed census.
///
/// Each call gets a fresh state dir, so nothing carries between tests.
fn census(server: &Server, args: &[&str]) -> (serde_json::Value, std::process::Output) {
    let state = std::env::temp_dir().join(format!(
        "retention-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let out = Command::new("python3")
        .arg(repo_root().join("containers/scripts/fleet-retention.py"))
        .args(args)
        .env("RETENTION_API_BASE", &server.base)
        .env("RETENTION_REGISTRY_BASE", &server.base)
        .env("RETENTION_NOW", NOW)
        .env("RETENTION_STATE", &state)
        .env("GH_TOKEN", "test-token")
        .env("REGISTRY", "ghcr.io/exgentic")
        .env("RETENTION_JOBS", "2")
        .output()
        .expect("run fleet-retention.py");
    let _ = std::fs::remove_dir_all(&state);
    let json = serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
    (json, out)
}

/// The routes a single-package fixture needs: the listing, that package's
/// versions, and a registry token endpoint.
fn routes(pkg: &str, versions: &str) -> Vec<(String, Vec<Reply>)> {
    vec![
        (
            "/orgs/exgentic/packages?".into(),
            vec![Reply(200, listing(&[pkg]))],
        ),
        (
            format!(
                "/orgs/exgentic/packages/container/{}/versions",
                urlencode(pkg)
            ),
            vec![Reply(200, versions.into())],
        ),
        (
            "/token".into(),
            vec![Reply(200, r#"{"token":"reg-token"}"#.into())],
        ),
    ]
}

fn urlencode(s: &str) -> String {
    s.replace('/', "%2F")
}

/// A version's verdict, derived from where it lands in the census.
fn verdict_of(pkg: &serde_json::Value, digest: &str) -> String {
    let aged = pkg["aged"].as_array().cloned().unwrap_or_default();
    if aged
        .iter()
        .any(|a| a["digest"] == format!("sha256:{digest}"))
    {
        return "aged".into();
    }
    "retained".into()
}

// ── the headline: reachability, not tags, decides ────────────────────────────

/// The real `benchmarks/aime` shape. `:latest` is an index whose four children
/// are one tagged per-arch image, one UNTAGGED per-arch image, and two UNTAGGED
/// attestation manifests. Every child is ancient, so age cannot save them —
/// only reachability can. A sweep that keyed on untagged-ness would delete the
/// arm64 half of the image everyone pulls.
#[test]
fn manifest_list_children_are_never_candidates() {
    let versions = format!(
        "[{},{},{},{},{}]",
        // the index itself, carrying `latest` + its hash tag
        version(1, "aaa", ANCIENT, &["latest", "89b8e419"]),
        version(2, "bbb", ANCIENT, &["latest-amd64"]), // tagged child
        version(3, "ccc", ANCIENT, &[]),               // UNTAGGED arm64 child
        version(4, "ddd", ANCIENT, &[]),               // UNTAGGED attestation
        version(5, "eee", ANCIENT, &[]),               // UNTAGGED attestation
    );
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(
            200,
            index(&[
                ("bbb", "linux/amd64", false),
                ("ccc", "linux/arm64", false),
                ("ddd", "unknown/unknown", true),
                ("eee", "unknown/unknown", true),
            ]),
        )],
    ));
    // The tagged per-arch child is a plain manifest, not an index.
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:bbb".into(),
        vec![Reply(
            200,
            r#"{"mediaType":"application/vnd.oci.image.manifest.v1+json"}"#.into(),
        )],
    ));
    let server = Server::start(r);
    let (census, out) = census(&server, &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let pkg = &census["packages"]["benchmarks/aime"];
    assert_eq!(
        pkg["aged"].as_array().map(Vec::len),
        Some(0),
        "no child of a live index may be a candidate, however old: {pkg}"
    );
    assert_eq!(pkg["counts"]["child"], 3, "3 untagged children: {pkg}");
    assert_eq!(pkg["counts"]["tagged"], 2);
    assert_eq!(pkg["reachable"], 5, "every version is reachable here");
    assert_eq!(census["totals"]["aged"], 0);
    for d in ["ccc", "ddd", "eee"] {
        assert_eq!(
            verdict_of(pkg, d),
            "retained",
            "untagged child {d} must be retained"
        );
    }
}

/// The attestation entry specifically. `fleet-status.sh` filters
/// `unknown/unknown` out to count platforms — right there, fatal here: deleting
/// an attestation manifest breaks `imagetools inspect` and provenance on a live
/// image. A copy-paste of that filter is the likeliest future regression, so it
/// gets its own assertion.
#[test]
fn attestation_manifests_are_reachable_not_garbage() {
    let versions = format!(
        "[{},{}]",
        version(1, "aaa", ANCIENT, &["latest"]),
        version(2, "att", ANCIENT, &[]), // untagged, unknown/unknown
    );
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(200, index(&[("att", "unknown/unknown", true)]))],
    ));
    let server = Server::start(r);
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["benchmarks/aime"];
    assert_eq!(
        pkg["aged"].as_array().map(Vec::len),
        Some(0),
        "an attestation manifest is reachable: {pkg}"
    );
    assert_eq!(pkg["counts"]["child"], 1);
}

/// Rules 18-20: a tag is a name a consumer can pull, so a tagged version is
/// never a candidate — not a hash tag, not `latest-arm64`, not a SemVer,
/// however old.
#[test]
fn a_tagged_version_is_never_a_candidate_however_old() {
    let hash = "89b8e419ddb29dc9b5de9a893850ab4678fe1f3c4e0595eea886be14cf1f9acc";
    let versions = format!(
        "[{},{},{}]",
        version(1, "aaa", ANCIENT, &[hash]),
        version(2, "bbb", ANCIENT, &["latest-arm64"]),
        version(3, "ccc", ANCIENT, &["v0.1.0"]),
    );
    let mut r = routes("benchmarks/aime", &versions);
    for d in ["aaa", "bbb", "ccc"] {
        r.push((
            format!("/v2/exgentic/benchmarks/aime/manifests/sha256:{d}"),
            vec![Reply(
                200,
                r#"{"mediaType":"application/vnd.oci.image.manifest.v1+json"}"#.into(),
            )],
        ));
    }
    let server = Server::start(r);
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["benchmarks/aime"];
    assert_eq!(pkg["counts"]["tagged"], 3);
    assert_eq!(
        pkg["aged"].as_array().map(Vec::len),
        Some(0),
        "a tag is a name someone can pull: {pkg}"
    );
    assert_eq!(pkg["verdict"], "clean");
}

// ── the 90-day window ───────────────────────────────────────────────────────

/// The boundary, from both sides. Cutoff is NOW - 90d = 2026-06-17.
#[test]
fn the_ninety_day_window_holds_from_both_sides() {
    // 89 days before NOW -> inside the window; 91 days -> outside.
    let inside = "2026-06-18T00:00:00Z";
    let outside = "2026-06-16T00:00:00Z";
    let versions = format!(
        "[{},{},{}]",
        version(1, "aaa", RECENT, &["latest"]),
        version(2, "bbb", inside, &[]),
        version(3, "ccc", outside, &[]),
    );
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(200, index(&[]))],
    ));
    let server = Server::start(r);
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["benchmarks/aime"];
    assert_eq!(
        pkg["counts"]["recent"], 1,
        "89d is inside the window: {pkg}"
    );
    assert_eq!(pkg["counts"]["aged"], 1, "91d is outside it: {pkg}");
    assert_eq!(verdict_of(pkg, "ccc"), "aged");
    assert_eq!(verdict_of(pkg, "bbb"), "retained");
    // The evidence a reviewer spot-checks a row by.
    let aged = &pkg["aged"][0];
    assert_eq!(aged["id"], 3);
    assert_eq!(aged["digest"], "sha256:ccc");
    assert!(aged["age_days"].as_i64().unwrap() >= 90);
    assert!(aged["because"].as_str().unwrap().contains("proxy"));
}

/// The census must never present its own clock as sound: rule 22 counts 90 days
/// from when a digest stopped being referenced, and the API records no such
/// field. Every manifest says so, in the artifact, not just in a log line.
#[test]
fn the_census_declares_its_clock_unsound() {
    let versions = format!("[{}]", version(1, "aaa", RECENT, &["latest"]));
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(200, index(&[]))],
    ));
    let server = Server::start(r);
    let (census, _) = census(&server, &[]);
    assert_eq!(census["clock"]["sound"], false);
    assert_eq!(census["clock"]["basis"], "created_at");
    assert_eq!(census["rule"], "delivery/RULES.md:22");
    assert_eq!(census["retention_days"], 90);
    let note = census["clock"]["note"].as_str().unwrap();
    assert!(note.contains("NOT"), "the note must be explicit: {note}");
}

// ── fail dirty ──────────────────────────────────────────────────────────────

/// Rule 14. A tagged manifest we could not read has UNKNOWN children, so the
/// reachable set is incomplete and every candidate in the package is suspect —
/// the whole package is withheld rather than nominating a digest that a
/// manifest we never read may point at. And the read must genuinely retry
/// first: GHCR drops enough connections that treating one as an answer is how a
/// sweep comes back smaller than the fleet.
#[test]
fn an_unreadable_manifest_blocks_its_whole_package() {
    let versions = format!(
        "[{},{}]",
        version(1, "aaa", ANCIENT, &["latest"]),
        // Would be `aged` on age alone — must be withheld instead.
        version(2, "old", ANCIENT, &[]),
    );
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(500, r#"{"message":"boom"}"#.into())],
    ));
    let server = Server::start(r);
    let (census, out) = census(&server, &[]);
    assert!(
        out.status.success(),
        "a blocked package is a report, not a crash"
    );
    let pkg = &census["packages"]["benchmarks/aime"];
    assert_eq!(pkg["verdict"], "blocked");
    assert_eq!(
        pkg["aged"].as_array().map(Vec::len),
        Some(0),
        "an incomplete reachable set nominates nothing: {pkg}"
    );
    // The tally goes with the list: a withheld candidate counted in the fleet
    // totals would report review candidates that no package's `aged` list
    // holds — a summary saying "2 to review" over an empty manifest.
    assert_eq!(
        census["totals"]["aged"], 0,
        "withheld, so not counted either"
    );
    assert_eq!(
        pkg["counts"]["unreadable"], 1,
        "the version whose status we could not determine is counted as such"
    );
    assert_eq!(census["blocked"][0], "benchmarks/aime");
    assert!(
        server.hits("/v2/exgentic/benchmarks/aime/manifests/sha256:aaa") >= 3,
        "a 5xx must be retried, not believed"
    );
}

/// The other side of the same discipline: a blip that clears must not poison
/// the package. Two 500s then a real index -> the children are found.
#[test]
fn a_transient_registry_error_retries_instead_of_blocking() {
    let versions = format!(
        "[{},{}]",
        version(1, "aaa", ANCIENT, &["latest"]),
        version(2, "ccc", ANCIENT, &[]),
    );
    let mut r = routes("benchmarks/aime", &versions);
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![
            Reply(500, r#"{"message":"blip"}"#.into()),
            Reply(500, r#"{"message":"blip"}"#.into()),
            Reply(200, index(&[("ccc", "linux/arm64", false)])),
        ],
    ));
    let server = Server::start(r);
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["benchmarks/aime"];
    assert_ne!(pkg["verdict"], "blocked", "the read cleared: {pkg}");
    assert_eq!(pkg["counts"]["child"], 1);
    assert_eq!(pkg["aged"].as_array().map(Vec::len), Some(0));
    assert_eq!(
        server.hits("/v2/exgentic/benchmarks/aime/manifests/sha256:aaa"),
        3,
        "it must have actually retried"
    );
}

// ── outside rule 22, and the questions the tool must not answer ─────────────

/// `buildcache/*` is BuildKit registry cache, not a published digest, so rule
/// 22's subject does not reach it — and nothing else authorises deleting it
/// either. Counted, never nominated. Pruning it needs a rule that says so.
#[test]
fn buildcache_is_counted_and_never_nominated() {
    let versions = format!(
        "[{},{}]",
        version(1, "aaa", ANCIENT, &["latest"]),
        version(2, "bbb", ANCIENT, &[]),
    );
    let server = Server::start(routes("buildcache/bases-amd64", &versions));
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["buildcache/bases-amd64"];
    assert_eq!(pkg["verdict"], "cache");
    assert_eq!(pkg["counts"]["cache"], 2);
    assert_eq!(
        pkg["aged"].as_array().map(Vec::len),
        Some(0),
        "cache is outside rule 22, not authorised for deletion: {pkg}"
    );
    assert_eq!(census["totals"]["cache"], 2);
    assert_eq!(census["totals"]["aged"], 0);
}

/// A package where every version is unreachable is usually one whose per-arch
/// images landed and whose manifest list was never stitched — the fix is
/// `fleet-tag.sh`, not deletion. The census reports the verdict and the tags it
/// does hold, so a reviewer can tell which it is; it does not decide.
#[test]
fn a_package_that_would_be_emptied_is_reported_not_decided() {
    let versions = format!(
        "[{},{}]",
        version(1, "aaa", ANCIENT, &[]),
        version(2, "bbb", ANCIENT, &[]),
    );
    let server = Server::start(routes("evals/gaia--pi", &versions));
    let (census, _) = census(&server, &[]);
    let pkg = &census["packages"]["evals/gaia--pi"];
    assert_eq!(pkg["verdict"], "would-empty");
    assert_eq!(census["would_empty"][0], "evals/gaia--pi");
    // The candidates are still listed — they are genuinely unreachable, and
    // suppressing them would substitute the tool's judgment for the reviewer's.
    assert_eq!(pkg["aged"].as_array().map(Vec::len), Some(2));
    assert!(pkg["all_tags"].is_array(), "the tags it holds are reported");
}

// ── the census must not overstate itself ────────────────────────────────────

/// The list endpoint rejects `per_page * page > 10000`, so a sweep cannot prove
/// it saw every package. That has to be a field in the artifact, which outlives
/// the run, not a line in a log nobody keeps.
#[test]
fn a_truncated_listing_is_declared_in_the_artifact() {
    // A full page every time: the script walks to the ceiling and stops.
    let full: Vec<&str> = (0..100).map(|_| "buildcache/x").collect();
    let routes = vec![
        (
            "/orgs/exgentic/packages?".into(),
            vec![Reply(200, listing(&full))],
        ),
        (
            "/orgs/exgentic/packages/container/buildcache%2Fx/versions".into(),
            vec![Reply(200, format!("[{}]", version(1, "aaa", RECENT, &[])))],
        ),
        (
            "/token".into(),
            vec![Reply(200, r#"{"token":"reg-token"}"#.into())],
        ),
    ];
    let server = Server::start(routes);
    let (census, out) = census(&server, &[]);
    assert!(
        out.status.success(),
        "truncation is a caveat, not a failure"
    );
    assert_eq!(census["completeness"]["truncated"], true);
    assert_eq!(census["completeness"]["api_ceiling"], 10000);
    let note = census["completeness"]["note"].as_str().unwrap();
    assert!(
        note.contains("CANNOT prove"),
        "note must be explicit: {note}"
    );
}

/// The ceiling and a broken endpoint both answer 500, so position decides which
/// it is. A 500 partway through the listing is a failed sweep, and calling it
/// "truncated" would publish a census that stopped early as merely capped —
/// exactly the fail-dirty case rule 14 forbids.
#[test]
fn a_mid_sweep_listing_error_fails_instead_of_reading_as_truncated() {
    // Page 1 answers a full page; page 2 — nowhere near the ceiling — 500s.
    let full: Vec<&str> = (0..100).map(|_| "buildcache/x").collect();
    let routes = vec![
        (
            "/orgs/exgentic/packages?package_type=container&per_page=100&page=1".into(),
            vec![Reply(200, listing(&full))],
        ),
        (
            "/orgs/exgentic/packages?package_type=container&per_page=100&page=2".into(),
            vec![Reply(500, r#"{"message":"Internal server error."}"#.into())],
        ),
        (
            "/token".into(),
            vec![Reply(200, r#"{"token":"reg-token"}"#.into())],
        ),
    ];
    let server = Server::start(routes);
    let (_, out) = census(&server, &[]);
    assert!(
        !out.status.success(),
        "a 500 short of the ceiling is a failure, not a truncation"
    );
    assert!(
        out.stdout.is_empty(),
        "no census may be emitted: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// A census assembled from a half-finished sweep would be reviewed as if
/// complete while its reachable sets were not, so it is never emitted: the run
/// says what is left and exits 3.
#[test]
fn a_partial_sweep_refuses_to_emit_a_census() {
    let versions = format!("[{}]", version(1, "aaa", RECENT, &["latest"]));
    let mut r = routes("benchmarks/aime", &versions);
    // A second package whose versions endpoint hard-fails with a 401: the
    // sweep cannot finish it, so no census may be printed.
    r[0] = (
        "/orgs/exgentic/packages?".into(),
        vec![Reply(
            200,
            listing(&["benchmarks/aime", "benchmarks/gsm8k"]),
        )],
    );
    r.push((
        "/orgs/exgentic/packages/container/benchmarks%2Fgsm8k/versions".into(),
        vec![Reply(401, r#"{"message":"Bad credentials"}"#.into())],
    ));
    r.push((
        "/v2/exgentic/benchmarks/aime/manifests/sha256:aaa".into(),
        vec![Reply(200, index(&[]))],
    ));
    let server = Server::start(r);
    let (_, out) = census(&server, &[]);
    assert!(
        !out.status.success(),
        "a sweep that could not finish must not print a census"
    );
    assert!(
        out.stdout.is_empty(),
        "nothing on stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `--only` narrows what is censused without widening what was listed, so a
/// one-package run is cheap and still honest about the universe it came from.
#[test]
fn only_narrows_the_census_to_matching_packages() {
    let versions = format!("[{}]", version(1, "aaa", ANCIENT, &[]));
    let mut r = routes("evals/gaia--pi", &versions);
    r[0] = (
        "/orgs/exgentic/packages?".into(),
        vec![Reply(
            200,
            listing(&["evals/gaia--pi", "benchmarks/aime", "agents/codex"]),
        )],
    );
    let server = Server::start(r);
    let (census, out) = census(&server, &["--only", "evals/*"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pkgs = census["packages"].as_object().unwrap();
    assert_eq!(pkgs.len(), 1, "only the matching package: {pkgs:?}");
    assert!(pkgs.contains_key("evals/gaia--pi"));
    assert_eq!(
        census["completeness"]["packages_listed"], 3,
        "the universe it came from is still reported"
    );
}
