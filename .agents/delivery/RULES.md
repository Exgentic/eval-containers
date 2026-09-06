# Delivery

**Status:** Active
**Date:** June 2026

## Abstract

How Eval Containers is published — the delivery-specific outcomes that refine the
one-version policy for the moment of release. One SemVer, set by the git tag,
already spans every image, the per-benchmark `eval-<benchmark>` compose
artifacts, the Helm chart, and the Rust CLI (top-level principle 9). These rules
govern how a single tag releases the
image fleet and the CLI together, which workflow owns which artifact, and the
gates that keep a release honest.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

1. **One tag, one release.** A release MUST be a single `vX.Y.Z` git tag that triggers both the image-fleet release and the CLI release.

2. **CLI release home.** The CLI release — binaries, installers, and the crates.io publish — MUST be produced by `.github/workflows/release.yml`.

3. **Fleet release home.** The image-fleet release — every fleet image and the per-benchmark `eval-<benchmark>` compose artifacts (one per benchmark) — MUST be produced by `.github/workflows/release-images.yml`.

4. **One Release owner.** A tag's GitHub Release object MUST be created and owned solely by the CLI release workflow.

5. **Tag-gated publishing.** The crate and any versioned image fleet MUST be published only by a `vX.Y.Z` tag push or an explicit `workflow_dispatch`, never by a branch push.

6. **Version-agreement gate.** A tagged release MUST abort unless the git tag equals both the `Cargo.toml` and `Chart.yaml` versions.

7. **Immutable crate versions.** A published crates.io version MUST NOT be reused or republished.

8. **Release-curated changelog.** `CHANGELOG.md` MUST be updated only when cutting a release tag.

9. **Standard sections only.** `CHANGELOG.md` MUST contain only the Keep a Changelog sections: Added, Changed, Deprecated, Removed, Fixed, and Security.

10. **Consumer-visible entries only.** A `CHANGELOG.md` entry MUST record a change visible to a consumer of a release.

11. **Build inputs.** An image's build inputs MUST comprise its build context, the build inputs of every in-repo base image, and the resolved digests of its external base images.

12. **Recorded inputs.** Every published image MUST record a hash of its build inputs in its image configuration.

13. **Carried-forward images.** A released image whose build inputs are unchanged from a prior release MUST be retagged from that release's digest rather than rebuilt.

14. **Fail dirty.** An image whose recorded build-input hash is absent, unreadable, or different from the repository's computed hash MUST be treated as changed, as MUST an image that is missing any platform it is expected to publish.

15. **Gate parity.** A carried-forward image MUST pass every release gate that a freshly built image passes.

16. **Continuous channel.** A push to the default branch MAY publish the `latest` fleet channel, and MUST publish only images whose build inputs changed.

17. **Verified publish.** A step that publishes an artifact MUST confirm the artifact is in the registry; a zero exit is not evidence. An artifact a stack cannot produce MUST be declared as such by the stack, never inferred from the text of a tool's error.

## References

- [Process](../RULES.md) — principle 9 (the one-version policy and version knobs); principle 13 (self-contained repo).
- [Repository, Naming & Compose](../compose/RULES.md) — rule 5 (version tags).
- [`release` skill](release/SKILL.md) — the procedure these outcomes constrain.
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) — the format principles 10–11 constrain `CHANGELOG.md` to.

## Changelog

| Date | Change |
|------|--------|
| 2026-06-11 | Initial version. Lifts the unified fleet + CLI release outcomes out of the root `RELEASE.md` into the delivery topic, which had skills but no `RULES.md`. |
| 2026-06-11 | Rule 5: permit an explicit `workflow_dispatch` (manual re-run with a version input) alongside a tag push — the fleet workflow's escape hatch; still forbids branch-push publishes. |
| 2026-06-14 | Added principles 8–10: the changelog is edited only when cutting a release tag, restricted to the Keep a Changelog sections, and limited to consumer-visible changes. |
| 2026-08-09 | Added rules 11–15 (build inputs, recorded inputs, carried-forward images, fail dirty, gate parity): a release carries an unchanged image's digest forward instead of rebuilding it, keyed on a recorded build-input hash that fails dirty and exempts nothing from release gates. Defining inputs to include resolved external-base digests makes an upstream base bump a *changed* input, so CVE refreshes rebuild naturally. Supersedes the judgment-based `skip_published` dispatch knob (#227); answers the silent-staleness objection that closed #241 (content hash, not path mapping) and implements the change-detection follow-up blessed in #168. |
| 2026-06-14 | Rule 3 + Abstract: the single shared `evaluate` compose artifact is replaced by per-benchmark `eval-<benchmark>` artifacts (one self-contained compose per benchmark, flattened at publish). A published artifact can't carry a dynamic per-benchmark `include:` (publish flattens includes), so per-benchmark sidecars (EnterpriseOps-Gym, WebArena, …) are baked in at publish and consumed with a single `-f`. |
| 2026-08-10 | Added rule 17 (Verified publish). Three bugs of one shape surfaced in a single release: arch-partial images read fresh, the merge silently skipped refs whose names it had mangled, and `docker compose publish` declined osworld's host-bind-mounted stack while exiting 0 — every one trusted a process instead of confirming the artifact. The second sentence closes the door the third came through: tau-bench's un-publishable stack was recognised by matching a substring of an error message, so a stack that failed *differently* (or silently) fell through. A stack now declares `x-eval-publish: false` itself. |
| 2026-08-10 | Rule 14 extended to platform completeness: an image missing an expected platform is *changed*, not fresh. A hash-only test cannot see a half-built image — when one arch's leaf fails, the merge still stitches a `:latest` from the surviving arch, whose config carries the matching hash, so the image reads fresh forever and the missing arch never self-retries (found when the first cold full-fleet rebuild left `benchmarks/appworld` amd64-only yet green). Expected platforms default to `linux/amd64,linux/arm64`; an image that is single-arch by necessity declares its own set with `LABEL eval.platforms`, read from the committed source rather than from the registry so a broken image cannot vouch for itself. |
| 2026-08-10 | Rule 5 rescoped from "the tagged image fleet" to "any versioned image fleet" — it gates *versioned* publishes, which a continuous `latest` publish is not. Added rule 16 (Continuous channel): a default-branch push MAY publish `latest`, and MUST publish only images whose build inputs changed — affordable exactly because rules 12–14 make "what changed" mechanical and rule 13 carries the rest forward. Versioned releases remain deliberate per rule 5; principle 9's "`latest` on `main`" becomes continuously true. |
