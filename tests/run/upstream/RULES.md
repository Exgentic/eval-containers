# Upstream reachability test rules

The upstream category probes **every pinned external reference** in
the fleet and fails if any has rotted away. Runs only in release
verification — it makes real network calls.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Release verification only.** This test is `#[ignore]` by default
   and MUST NOT run in contribution verification (parent rule 1.1
   forbids network calls).

2. **Metadata-only probes.** The test MUST use `curl -I` / `HEAD` for
   URLs and `docker manifest inspect` for registry references. It MUST
   NOT pull, download, or materialize anything to disk. This keeps it
   a static validation under parent rule 6a — no testcontainers
   dependency.

3. **No false positives from known-broken.** Benchmarks listed in
   `tests/build/known-broken.md` are excluded. If the upstream is
   already documented as gated/unreachable, re-reporting it here is
   noise.

## What to probe

4. **`LABEL eval.benchmark.upstream_base`** — every benchmark that
   inherits from a third-party image declares this label. `docker
   manifest inspect` MUST succeed for the pinned tag.

5. **`FROM` lines** — every non-`scratch`, non-`${EVAL_TASK_ID}`
   interpolated `FROM` MUST resolve via `docker manifest inspect`.

6. **HuggingFace + GitHub raw URLs** — every RUN line that fetches
   from `huggingface.co/datasets/` or `raw.githubusercontent.com`
   MUST resolve (HEAD 2xx/3xx).

## Rate limits

7. **Be nice to upstream.** The sweep MUST sleep at least 50 ms between
   probes. HuggingFace and GitHub both rate-limit aggressive scans.

8. **Timeouts.** Every probe MUST have a 20-second timeout. A probe
   that hangs forever is a harder failure mode than a probe that
   returns 404.

## Failure policy

9. **Any 404 is red.** A pinned reference that has disappeared blocks
   the release. Fix the pin (update to a new revision, mirror the
   asset locally, or mark the benchmark known-broken with a citation).

10. **Auth walls are yellow, not red.** A 401/403 on a gated dataset
    means the release runner lacks the credential. That's a runner
    config issue, not upstream drift — the entry graduates into
    `build/known-broken.md` with a citation.
