# Release freshness

*Concept · for operators · derives from [`.agents/delivery/RULES.md`](../../.agents/delivery/RULES.md) rules 11–15, [`.agents/RULES.md`](../../.agents/RULES.md) principle 9.*

Every fleet image carries a label, `eval.input-hash`, recording a hash of its
build inputs: the git tree of its build context and of every in-repo base it
builds on, plus (for per-task images) the task id. The hash is a pure function
of the repository at a commit — computable offline, identically, by anyone:

```bash
containers/scripts/fleet-hash.sh                  # every image's expected hash
containers/scripts/fleet-status.sh v0.2.0         # compare a published tag
```

`fleet-status` classifies every image at a tag as **fresh** (recorded hash
matches the repo), **stale**, **unlabeled**, or **absent** — everything
non-fresh counts as *changed*, and absent/unreadable fails dirty rather than
fresh. The dispatchable **Fleet status** workflow runs the same sweep against
any tag and summarizes the fleet's freshness in one page.

## What a version tag means

Image builds are not bit-reproducible (package resolution moves under
identical inputs), so rebuilding an *unchanged* image would silently ship
different bits under the new version. The release therefore does the safer
thing: **an image whose inputs are unchanged from the prior release is
retagged from that release's digest — same bits, new tag — instead of being
rebuilt.** A version tag pins a *coherent, tested set of inputs*, not a build
timestamp; two consecutive versions may share digests for images whose inputs
did not change, and each image's SLSA provenance honestly names the run that
actually built it.

Only images work this way. Artifacts that embed the version in their own bytes
— the per-benchmark `eval-<benchmark>` compose artifacts, the Helm chart, the
CLI — are republished fresh on every release by definition.

## The continuous `latest` channel

Every push to `main` publishes `:latest` automatically — and publishes **only
the push's delta**. The release workflow computes the stale set up front and
prunes every build matrix to it: a push touching one benchmark builds that
benchmark (and its combos), a push touching a shared base rebuilds the base
and everything the cascade reaches, and a push touching nothing under
`containers/` publishes nothing. Versioned `vX.Y.Z` releases remain separate,
deliberate, tag-triggered events — they always process the full fleet so every
image gains the new tag (built or carried forward), pass the CVE gate, and are
what consumers should pin. Per-task images (~600 heavy builds) are excluded
from the continuous channel and refresh only on versioned releases or manual
dispatch.

## Forcing a rebuild

The input hash sees the repository, not the outside world: an upstream base
image (`python:3.12-slim`) or unpinned package moving does not change any
input. To pick up upstream fixes, dispatch **Release the fleet** with
`force_rebuild: true` (or `rebuild_bases: true` for the shared bases alone) —
principle 9 classes such CVE/base refreshes as a patch release. The CVE gate
scans whatever the release tag points to, carried-forward or freshly built, so
a stale-but-carried base cannot slip through a gated release unscanned.
