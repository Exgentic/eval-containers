---
name: audit-rules-drift
description: >-
  Walk every normative RULES principle against the actual repository state and
  report, per rule, whether the code still honours it — compliant, partial
  drift, or drifted — with file:line evidence and a proposed mechanical check for
  each gap. Use this for "audit RULES drift", "do the rules still match the
  code", before a release (step 29 of the `verify` walk), or after a RULES rename
  that the code may not have followed. This is the rule-text-vs-code consistency
  pass; for the whole-repo build/fixture/count audit use audit-fleet, and to
  apply a single topic's rules to one artifact use the meta `review` skill.
---

# Audit RULES drift

A rule stated only in prose, with no mechanical check and no walked audit, is a
comment, not a rule (`.agents/verification/RULES.md:13`). Rule text and code
drift apart in two directions: the code changes and the rule text goes stale
(e.g. a `TASK_ID` → `EVAL_TASK_ID` rename that never reached the RULES body), or
the rule mandates a behaviour the code never implemented (e.g. an entrypoint
that "MUST read `EVAL_AGENT_VERSION`" but does not). This audit walks every
normative principle against the tree and classifies each one, with evidence, so
the gaps are visible before a release and so each gap can be promoted to a
mechanical check.

The audit is toolchain-agnostic: a human greps the tree and reads the rule
bodies, or a sub-agent does the same. The output format is fixed — one entry per
rule, a verdict, file:line evidence, a suggested fix — so reports diff across
releases.

## Rules this skill serves

- `.agents/verification/RULES.md:13` — mechanical > procedural > aspirational;
  this audit exists to find aspirational rules (prose with no check) and either
  confirm them against the code or convert them into a mechanical check.
- `tests/static/RULES.md:9` — prefer mechanical over procedural;
  every drift finding ends in a proposed catalog rule so the same drift cannot
  recur unseen.
- `tests/static/RULES.md:10` — rule IDs must stay aligned across
  RULES bodies and the rule catalogs; this audit catches the divergence when
  text and code disagree.
- `.agents/meta/rules/RULES.md` and `.agents/meta/skills/RULES.md` — the meta
  that governs what a rule and a skill are; a drift finding may be that a rule's
  own text no longer states a checkable outcome.

## Procedure

Input is the whole working tree at a given commit. Context: the current RULES
bodies (top-level and every `.agents/<area>/RULES.md`) and the previous drift
report (what was flagged last time).

1. **Enumerate the normative rules.** List every numbered principle in the
   top-level RULES and in each `.agents/<area>/RULES.md` that asserts a MUST /
   MUST NOT outcome about the code or artifacts. WHY: the audit is per-rule;
   skipping a rule hides the drift it would have caught.

2. **For each rule, locate its evidence in the code.** Grep the tree for the
   concrete thing the rule mandates — an env var name, a label key, a filename
   convention, an entrypoint behaviour, a compose field. Read the matching
   `file:line`. WHY: a verdict without file:line evidence is an opinion; the
   report must let the next reader re-check it cheaply.

3. **Classify each rule's verdict.** WHY: the three states separate "ship it"
   from "fix the text" from "fix the code or retract the rule".
   - **compliant** — the code honours the rule. Note whether it is already
     enforced mechanically (cite the catalog rule/test) or only by convention.
   - **partial drift** — the rule's intent holds but the text is stale,
     ambiguous, or carves out an unstated exception (e.g. a rule contradicts
     another, or a "MUST X or equivalent" where only the weaker "equivalent" is
     ever used).
   - **drifted** — the code does not do what the rule says (stale rename the
     code followed but the text did not; or a mandated behaviour that no image
     implements). This is the highest-severity finding.

4. **For every partial or drifted rule, write a suggested fix.** Each fix names
   the concrete edit: either tighten/retract the rule text, or implement the
   missing behaviour in the named file. WHY: a drift finding without a decision
   ("reword the rule" vs "implement the path") just re-surfaces next cycle.

5. **For every mechanizable drift, propose a new catalog rule.** State the rule
   ID, severity, and the predicate (e.g. "assert `core/runner/run`
   references `EVAL_BENCHMARK_VERSION` and writes `/output/task/version.json`";
   "assert every `models/*/Dockerfile` contains `LABEL eval.model.litellm_version=`";
   "assert no benchmark `compose.yaml` uses `EVAL_*_VERSION` as a Docker image
   tag"; "assert every file under `tests/run/replay/fixtures/` ends in
   `.traces.jsonl`"). WHY: this is the mechanical > procedural escalation
   (`tests/static/RULES.md:9`) — a drift that recurs belongs in
   code, not in a quarterly manual walk.

6. **Emit one report** in the fixed structure: a per-rule section (verdict,
   evidence, suggested fix), a summary count (compliant / partial / drifted),
   the top drift findings, and the proposed mechanical checks. WHY: the fixed
   shape lets the report diff cleanly and feeds question 9 of the `audit-fleet`
   walk. Skeleton:

   ```
   # Rules Drift Audit — YYYY-MM-DD
   Commit: <sha>

   ## Per-rule findings

   ### RULES.md principle 9 — Pin by default (tag vs. version)
   - Verdict: ✗ drifted (systemic)
   - Evidence: 95 of 96 per-benchmark compose files use `${EVAL_AGENT_VERSION:-latest}`
     as the image tag (e.g. benchmarks/aime/compose.yaml:17); zero reference
     EVAL_AGENT_TAG. No agent entrypoint reads EVAL_AGENT_VERSION.
   - Suggested fix: implement the version-override path and rewrite compose to
     use EVAL_AGENT_TAG, or retract the two-knob split from principle 9.

   ...

   ## Summary
   - ✓ compliant: N
   - ⚠ partial drift: N
   - ✗ drifted: N

   ## Top drift findings
   - ...

   ## Proposed mechanical checks
   - new rule `entrypoint_reads_benchmark_version`: ...
   ```

## When to run

- Before cutting a release — step 29 of the `verify` skill (RULES still match
  the repo).
- After any RULES rename or restructure that the code may not have followed.
- Quarterly drift sweep, alongside `audit-fleet`.
- When a new mechanical catalog rule lands (confirm its rule text and ID still
  match the code it checks).

## References

- `.agents/verification/RULES.md` — the precedence rule (mechanical >
  procedural > aspirational) this audit enforces.
- `tests/static/RULES.md` — where every mechanizable drift
  finding should land as a catalog rule.
- `.agents/meta/rules/RULES.md`, `.agents/meta/skills/RULES.md` — what a rule
  and a skill must be.
- `.agents/verification/audit-fleet/SKILL.md` — its question 9 (do the RULES
  principles hold?) is answered by this audit.
