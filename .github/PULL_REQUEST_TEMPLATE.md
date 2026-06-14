<!--
Thank you for contributing to Eval Containers.

This is the general PR template. If your PR adds a new **benchmark**
or a new **agent**, please use the typed template instead:

- New benchmark: ?template=benchmark.md
- New agent:     ?template=agent.md

(append to the PR URL after opening it in draft state, or open the
PR with `gh pr create --template ...`)

For PRs that modify existing code, fill in the sections below.
-->

> This repo is governed by [`AGENTS.md`](../AGENTS.md) and [`.agents/`](../.agents/). Check your change against the rules for the area you touched; for a release, follow [`.agents/verification/verify/SKILL.md`](../.agents/verification/verify/SKILL.md).
>
> **Do not edit `CHANGELOG.md`** — it is release-curated, not per-PR ([`.agents/delivery/RULES.md`](../.agents/delivery/RULES.md) 8–10).

## Summary

<!-- One paragraph: what changed and why -->

## Type of change

- [ ] Bug fix
- [ ] New feature (not a new benchmark/agent — use the typed templates for those)
- [ ] Refactor / cleanup
- [ ] Documentation only
- [ ] CI / tooling
- [ ] Breaking change (specify which rule or contract changed)

## Verification

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (sanity gates: check, compose, dockerfile_inspection, task_inspection, upstream unit tests)
- [ ] If the change touches a Dockerfile or compose file: one affected benchmark/agent builds locally (`docker build` or `eval-containers build bench <name>`)
- [ ] If the change touches mechanical rule catalogs: every new rule has a unit test for the positive and negative case
- [ ] If the change touches shared infrastructure (core/entrypoint, core/litellm, core/combination.Dockerfile): one smoke run against `aime` or `mmlu` via the live driver passes

## RULES.md impact

<!-- If this PR changes the shape of an artifact or the meaning of an existing rule, update the relevant RULES.md and the changelog at the bottom of that file. -->

- [ ] No RULES.md update needed
- [ ] RULES.md updated: <!-- which file? which rule? changelog entry? -->

## Breaking changes

<!-- If this PR breaks the contribution verification pipeline for existing contributors, describe the migration path. -->

- [ ] No breaking changes
- [ ] Breaking change documented above
