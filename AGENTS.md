# AGENTS.md

Eval Containers is governed by **`.agents/`** — a self-contained body of
**rules** (what a result must be) and **skills** (how to produce it), governed
by its own meta. Before you change anything here, read the doctrine for the
area you're touching and treat its rules as binding: code that violates an
active rule must not merge.

## How the doctrine works

- A **rule** states an *outcome* a finished artifact must have. Rules are
  normative.
- A **skill** is a *procedure* you can follow to produce a conforming result.
- What a rule and a skill must be — their form, placement, and lifecycle — is
  fixed by the meta in [`.agents/meta/`](.agents/meta/). This directory is
  standalone: it governs itself.

## Map

**Meta (the constitution)**
- [`.agents/meta/rules/RULES.md`](.agents/meta/rules/RULES.md) — what a rule is, and how rules are written.
- [`.agents/meta/skills/RULES.md`](.agents/meta/skills/RULES.md) — what a skill is, and how skills are written.

**Rules — the *what***
- [`.agents/RULES.md`](.agents/RULES.md) — project principles
- [`.agents/benchmarks/RULES.md`](.agents/benchmarks/RULES.md) · [`.agents/agents/RULES.md`](.agents/agents/RULES.md) · [`.agents/models/RULES.md`](.agents/models/RULES.md)
- [`.agents/compose/RULES.md`](.agents/compose/RULES.md) · [`.agents/gateways/RULES.md`](.agents/gateways/RULES.md) · [`.agents/src/RULES.md`](.agents/src/RULES.md)
- [`.agents/verification/RULES.md`](.agents/verification/RULES.md) — testing strategy; per-category rules live beside their tests in `tests/<category>/RULES.md` (paired with the enforcing Rust) and are indexed from the strategy.
- [`.agents/verification/audit/RULES.md`](.agents/verification/audit/RULES.md) — the `AUDIT.md` reports (per-benchmark + a generated project-level rollup) across validity / safety / size / speed / cost, with honest statuses.
- [`.agents/docs/RULES.md`](.agents/docs/RULES.md) — the human-facing `docs/` site: doctrine governs, docs explain.

**Skills — the *how***
- Add a component — [`benchmarks/add-benchmark`](.agents/benchmarks/add-benchmark/SKILL.md) · [`agents/add-agent`](.agents/agents/add-agent/SKILL.md)
- Build & release — [`delivery/build`](.agents/delivery/build/SKILL.md) · [`delivery/release`](.agents/delivery/release/SKILL.md)
- Verify & audit — [`verification/verify`](.agents/verification/verify/SKILL.md) · [`audit-dockerfile`](.agents/verification/audit-dockerfile/SKILL.md) · [`audit-trajectory`](.agents/verification/audit-trajectory/SKILL.md) · [`audit-fleet`](.agents/verification/audit-fleet/SKILL.md) · [`audit-rules-drift`](.agents/verification/audit-rules-drift/SKILL.md) · [`audit/audit-benchmark`](.agents/verification/audit/audit-benchmark/SKILL.md) · [`audit/audit-rollup`](.agents/verification/audit/audit-rollup/SKILL.md)

## Working in this repo

1. **Find the topic(s)** your change touches and read their `RULES.md` (and any ancestor topic's, up to `.agents/RULES.md`).
2. **If a skill exists** for what you're doing (adding a benchmark/agent, building, releasing, auditing), follow it.
3. **Before opening a PR**, check your change against the relevant rules — `.agents/verification/verify/SKILL.md` is the release walk.
4. **Changing a rule** is a doctrine change: edit the rule in its one home, add a Changelog entry, and cite it in your PR. Don't encode new standards anywhere but `.agents/`.
