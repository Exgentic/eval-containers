---
name: sweep-retention
description: >-
  Take and review the registry retention census — what every published version
  is, and which ones nothing resolves to any more. Use when auditing what
  ghcr.io/exgentic holds, when asked whether stale versions can be cleaned up,
  or before proposing any registry deletion. It deletes nothing: the census is
  the artifact a human reviews. For reclaiming local Docker disk use
  `eval-containers prune`; for whether images are stale against the repo use
  the `fleet-status.sh` freshness report.
---

# Sweep the registry retention census

The registry holds ~9,900 packages and ~1.5M versions, and rule 22 says a
published digest is retained while `latest`, a release, or a published artifact
resolves to it, and for 90 days after that ceases. This skill produces the
census that makes the question answerable, and tells you how to read it.

Serves: `.agents/delivery/RULES.md:22` (retention), `:18`–`:20` (hash tag,
immutability, the `latest` alias), `:14` (fail dirty).

**Nothing here deletes anything.** The tool has no delete path and the workflow
has no `packages: delete`. Deletion is a separate, unbuilt step, and step 5
explains why it cannot be built from this census alone.

1. **Take the census.** Weekly in CI (`.github/workflows/fleet-retention.yml`,
   report-only), or locally for a subset:

   ```bash
   GH_TOKEN=$(gh auth token) \
     containers/scripts/fleet-retention.py --only 'benchmarks/*' > census.json
   ```

   Why a subset locally: the whole registry is ~20k paginated requests against a
   5,000/hr budget shared with the release pipeline. The sweep parks itself at a
   floor and resumes per package, so an interrupted run loses no work — re-run
   the same command to continue. Exit 3 means "partially enumerated, re-run",
   not failure.

2. **Read the verdicts.** Every version gets exactly one. Only `aged` is a
   review candidate; everything else is retained and needs no action.

   | verdict | what it means |
   |---|---|
   | `tagged` | carries a tag — a name someone can pull (rules 18–20) |
   | `child` | untagged, but a child of a tagged index — per-arch **or** attestation |
   | `recent` | nothing resolves to it, but it is inside the 90-day window |
   | `aged` | nothing resolves to it and it predates the cutoff |
   | `cache` | a `buildcache/*` version, outside rule 22 |
   | `unreadable` | a tagged manifest never read, so reachability is unknown |

   Why `child` is the verdict that matters: **an untagged version is not a
   dangling one.** A multi-arch image is an index whose per-arch and attestation
   children are themselves versions carrying no tags of their own. On
   `benchmarks/aime`, 3 of the 4 children of the live `:latest` are untagged — so
   "delete the untagged versions" deletes the arm64 half of the image everyone
   pulls, and the attestations that make `imagetools inspect` and provenance
   work. Never reason about untagged-ness; reason about the `child` count.

3. **Treat `aged` as a census, never a clearance.** Check
   `clock.sound` in the output — it is always `false`. Rule 22 counts 90 days
   from when a digest stopped being *referenced*, and the packages API records no
   such field (`updated_at` equals `created_at` on every version measured), so
   `aged` is keyed on creation date instead. That over-selects: a digest created
   100 days ago may have held `latest` until yesterday and still reads `aged`.
   An `aged` row is a thing to investigate, not a thing cleared for deletion.

4. **Act on the verdicts that have an action today.**

   - **`would-empty`** — every version in the package is a candidate, which
     means nothing in it is tagged at all. That is usually a build whose
     per-arch images landed and whose manifest list was never stitched, so check
     the reported `all_tags` for `latest-amd64` / `latest-arm64`: if they are
     there, the fix is `containers/scripts/fleet-tag.sh`, not deletion. Restoring
     a pullable `latest` is worth more than reclaiming the space.
   - **`blocked`** — a tagged manifest could not be read, so the package's
     reachable set is incomplete and its candidates were withheld (rule 14, fail
     dirty). Re-run the sweep for that package; a persistent block is a registry
     problem worth its own issue.
   - **`cache`** — reported and counted, never nominated. `buildcache/*` is
     BuildKit registry cache, not a published digest, so rule 22's subject does
     not reach it — but nothing else authorises deleting it either. Pruning it
     needs a new rule; propose one as a rules-only change (contributing rule 2)
     with the census as the evidence.

5. **Do not improvise a deletion.** Two things must land before a delete path
   can honour rule 22, and both are rule changes, not code:

   - the retention clock must become observable — the publisher recording
     supersession when it moves a tag, so "unreferenced for 90 days" is a fact
     rather than an inference from creation dates;
   - `.agents/delivery/RULES.md:21` must actually hold. Published
     `eval-<benchmark>` artifacts currently default their image references to
     `:latest`, not to hash tags, so the set of hash tags a published artifact
     pins — the third thing rule 22 protects — cannot be enumerated yet.

   Until then the honest output is a census and a review, which is what this
   skill produces. A sweep keyed on age alone would delete digests rule 22
   protects.

6. **Record what you found.** The CI run keeps `retention.json` as an artifact
   for 90 days and writes the verdict table to the run summary; that is the
   recorded sweep. For a local run, attach the census to the issue or PR that
   prompted it, and cite the totals rather than re-deriving them — the census is
   the shared artifact, so nobody re-scans the registry to check your numbers.
