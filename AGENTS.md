# AGENTS.md

Eval Containers is governed by **`doctrine/`** — a self-contained body of
**rules** (what a result must be) and **skills** (how to produce it), governed
by its own meta. Before you change anything here, read the doctrine for the
area you're touching and treat its rules as binding: code that violates an
active rule must not merge.

## How the doctrine works

- A **rule** states an *outcome* a finished artifact must have. Rules are
  normative.
- A **skill** is a *procedure* you can follow to produce a conforming result.
- What a rule and a skill must be — their form, placement, and lifecycle — is
  fixed by the meta in [`doctrine/meta/`](doctrine/meta/). This directory is
  standalone: it governs itself.

## Map

**Meta (the constitution)**
- [`doctrine/meta/rules/RULES.md`](doctrine/meta/rules/RULES.md) — what a rule is, and how rules are written.
- [`doctrine/meta/skills/RULES.md`](doctrine/meta/skills/RULES.md) — what a skill is, and how skills are written.

**Rules — the *what***
- [`doctrine/RULES.md`](doctrine/RULES.md) — project principles · [`doctrine/MANIFESTO.md`](doctrine/MANIFESTO.md)
- [`doctrine/benchmarks/RULES.md`](doctrine/benchmarks/RULES.md) · [`doctrine/agents/RULES.md`](doctrine/agents/RULES.md) · [`doctrine/models/RULES.md`](doctrine/models/RULES.md)
- [`doctrine/compose/RULES.md`](doctrine/compose/RULES.md) · [`doctrine/gateways/RULES.md`](doctrine/gateways/RULES.md) · [`doctrine/src/RULES.md`](doctrine/src/RULES.md)
- [`doctrine/verification/RULES.md`](doctrine/verification/RULES.md) — testing strategy; per-category rules live beside their tests in `tests/<category>/RULES.md` (paired with the enforcing Rust) and are indexed from the strategy.
- [`doctrine/verification/audit/RULES.md`](doctrine/verification/audit/RULES.md) — the `AUDIT.md` reports (per-benchmark + a generated project-level rollup) across validity / safety / size / speed / cost, with honest statuses.
- [`doctrine/docs/RULES.md`](doctrine/docs/RULES.md) — the human-facing `docs/` site: doctrine governs, docs explain.

**Skills — the *how***
- Add a component — [`benchmarks/add-benchmark`](doctrine/benchmarks/add-benchmark/SKILL.md) · [`agents/add-agent`](doctrine/agents/add-agent/SKILL.md)
- Build & release — [`delivery/build`](doctrine/delivery/build/SKILL.md) · [`delivery/release`](doctrine/delivery/release/SKILL.md)
- Verify & audit — [`verification/verify`](doctrine/verification/verify/SKILL.md) · [`audit-dockerfile`](doctrine/verification/audit-dockerfile/SKILL.md) · [`audit-trajectory`](doctrine/verification/audit-trajectory/SKILL.md) · [`audit-fleet`](doctrine/verification/audit-fleet/SKILL.md) · [`audit-rules-drift`](doctrine/verification/audit-rules-drift/SKILL.md) · [`audit/audit-benchmark`](doctrine/verification/audit/audit-benchmark/SKILL.md) · [`audit/audit-rollup`](doctrine/verification/audit/audit-rollup/SKILL.md)

## Working in this repo

1. **Find the topic(s)** your change touches and read their `RULES.md` (and any ancestor topic's, up to `doctrine/RULES.md`).
2. **If a skill exists** for what you're doing (adding a benchmark/agent, building, releasing, auditing), follow it.
3. **Before opening a PR**, check your change against the relevant rules — `doctrine/verification/verify/SKILL.md` is the release walk.
4. **Changing a rule** is a doctrine change: edit the rule in its one home, add a Changelog entry, and cite it in your PR. Don't encode new standards anywhere but `doctrine/`.
