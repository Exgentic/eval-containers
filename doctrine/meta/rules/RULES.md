# Meta Rules

**Status:** Active
**Date:** May 2026

## Abstract

Eval Containers governs itself through `doctrine/`: a self-contained body of
**rules** (what a result must be) and **skills** (how to produce it). This
document specifies what a rule is, the form every `RULES.md` under `doctrine/`
must take, where rules live, and how they change. Skills are governed by
`doctrine/meta/skills/RULES.md`.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in every
`RULES.md` under `doctrine/` are to be interpreted as described in BCP 14,
RFC 2119 and RFC 8174.

A *rule* constrains an outcome — a property observable in a finished artifact.
A *skill* describes a method — a procedure for producing an artifact, governed
by `doctrine/meta/skills/RULES.md`. A *topic* is a directory under `doctrine/`
grouping the rules and skills for one area.

## Principles

1. **Rules are normative.** Code that violates an active rule **MUST NOT** be
   merged.

2. **Rules govern outcomes, not methods.** A rule **MUST** describe what must
   be true of a finished artifact, never the procedure to achieve it.

3. **Governance is centralized in `doctrine/`, with one exception.** Every
   cross-cutting rule and skill **MUST** live under `doctrine/`, except a rule
   that is the human-readable half of a code-paired catalog beside
   `tests/<category>/`, which **MUST** stay beside that code and be linked from
   `doctrine/verification/RULES.md`.

4. **One home per rule.** Each rule **MUST** appear in exactly one `RULES.md`
   and **MUST NOT** be mirrored into a more specific topic.

5. **Topics may nest.** A topic's rules **MUST** live in
   `doctrine/<topic>/RULES.md`, and a nested rule **MUST** govern over a
   conflicting ancestor rule.

6. **Format.** Every `RULES.md` **MUST** contain, in order: a title, a Status,
   a Date, an Abstract, a Terminology section citing RFC 2119 when it uses the
   keywords, numbered normative requirements, a References section, and a
   Changelog.

7. **Requirements are addressable.** Each numbered requirement **MUST** be
   citable from anywhere in the tree as `doctrine/<topic>/RULES.md:<n>`.

8. **Status lifecycle.** Each `RULES.md` **MUST** declare a status of Draft,
   Active, or Superseded.

9. **Changelog required.** Every change to an active `RULES.md` **MUST** be
   recorded in its Changelog with a date and a summary.

10. **Revision, not silent drift.** A published requirement **MUST NOT** be
    silently removed or renumbered, but **MUST** be deprecated in place with a
    replacement reference where one applies.

11. **Concise.** Each requirement **MUST** use the fewest words that stay
    unambiguous.

12. **Atomic.** Each numbered requirement **MUST** be a single sentence stating
    one prescription with one keyword.

13. **Example-free.** A requirement **MUST NOT** contain examples; illustration
    belongs in a skill or in `docs/`.

14. **Bounded abstract.** An Abstract **MUST** be one paragraph of at most 80
    words.

## References

- RFC 2119, RFC 8174 (BCP 14).
- `doctrine/meta/skills/RULES.md` — the companion meta for skills.
- `AGENTS.md` — the repository's entry point into `doctrine/`.

## Changelog

| Date       | Change                                                                  |
|------------|-------------------------------------------------------------------------|
| 2026-05-31 | Initial version. Centralizes the former distributed rules graph under `doctrine/`, adds the rule/skill split, and replaces the "rules live next to the code" principle. |
| 2026-06-03 | Added principles 11–14 (Concise, Atomic, Example-free, Bounded abstract) to cap rule length and verbosity. Pre-existing principles 1–10 and sibling `RULES.md` files predate these and need a follow-up tightening pass. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
