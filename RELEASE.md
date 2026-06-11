# Release & Versioning

Eval Containers ships as **one product with one SemVer**: a single `vX.Y.Z` git
tag publishes the image fleet, the `evaluate` compose, the Helm chart, and the
Rust CLI at the same version. The guiding principle: **CI builds the fleet;
humans build one thing at a time.**

This file is an entry point, not the policy — the canonical homes are in the
doctrine:

- **Versioning policy** — [`doctrine/RULES.md`](doctrine/RULES.md) principle 9
  (one version; the two orthogonal version knobs) and
  [`doctrine/compose/RULES.md`](doctrine/compose/RULES.md) rule 5 (version tags).
- **Release outcomes** — [`doctrine/delivery/RULES.md`](doctrine/delivery/RULES.md):
  one tag releases both the fleet and the CLI, which workflow owns which artifact
  (`release.yml` = CLI + the GitHub Release; `release-images.yml` = fleet →
  registry), and the version-agreement and crate-immutability gates.
- **Release procedure** — the
  [`release` skill](doctrine/delivery/release/SKILL.md) and its
  [readiness checklist](doctrine/delivery/release/references/readiness.md).
- **Release notes** — [CHANGELOG.md](CHANGELOG.md).
