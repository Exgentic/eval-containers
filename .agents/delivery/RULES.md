# Delivery

**Status:** Active
**Date:** June 2026

## Abstract

How Eval Containers is published — the delivery-specific outcomes that refine the
one-version policy for the moment of release. One SemVer, set by the git tag,
already spans every image, the `evaluate` compose, the Helm chart, and the Rust
CLI (top-level principle 9). These rules govern how a single tag releases the
image fleet and the CLI together, which workflow owns which artifact, and the
gates that keep a release honest.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

1. **One tag, one release.** A release MUST be a single `vX.Y.Z` git tag that triggers both the image-fleet release and the CLI release.

2. **CLI release home.** The CLI release — binaries, installers, and the crates.io publish — MUST be produced by `.github/workflows/release.yml`.

3. **Fleet release home.** The image-fleet release — every fleet image and the `evaluate` compose artifact — MUST be produced by `.github/workflows/release-images.yml`.

4. **One Release owner.** A tag's GitHub Release object MUST be created and owned solely by the CLI release workflow.

5. **Tag-gated publishing.** The crate and the tagged image fleet MUST be published only by a `vX.Y.Z` tag push or an explicit `workflow_dispatch`, never by a branch push.

6. **Version-agreement gate.** A tagged release MUST abort unless the git tag equals both the `Cargo.toml` and `Chart.yaml` versions.

7. **Immutable crate versions.** A published crates.io version MUST NOT be reused or republished.

8. **Release-curated changelog.** `CHANGELOG.md` MUST be updated only when cutting a release tag.

9. **Standard sections only.** `CHANGELOG.md` MUST contain only the Keep a Changelog sections: Added, Changed, Deprecated, Removed, Fixed, and Security.

10. **Consumer-visible entries only.** A `CHANGELOG.md` entry MUST record a change visible to a consumer of a release.

## References

- [Process](../RULES.md) — principle 9 (the one-version policy and version knobs); principle 13 (self-contained repo).
- [Repository, Naming & Output](../compose/RULES.md) — rule 5 (version tags).
- [`release` skill](release/SKILL.md) — the procedure these outcomes constrain.
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) — the format principles 10–11 constrain `CHANGELOG.md` to.

## Changelog

| Date | Change |
|------|--------|
| 2026-06-11 | Initial version. Lifts the unified fleet + CLI release outcomes out of the root `RELEASE.md` into the delivery topic, which had skills but no `RULES.md`. |
| 2026-06-11 | Rule 5: permit an explicit `workflow_dispatch` (manual re-run with a version input) alongside a tag push — the fleet workflow's escape hatch; still forbids branch-push publishes. |
| 2026-06-14 | Added principles 8–10: the changelog is edited only when cutting a release tag, restricted to the Keep a Changelog sections, and limited to consumer-visible changes. |
