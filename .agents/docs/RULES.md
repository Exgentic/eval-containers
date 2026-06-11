# Documentation Rules

**Status:** Active
**Date:** June 2026

## Abstract

`docs/` is the human-facing documentation site; `.agents/` is the normative
rulebook. Docs explain the system and show how to use it; doctrine fixes what
must be true. Where the two conflict, doctrine governs. This document fixes what
`docs/` must contain, how it is structured, and the bar each page must clear.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in RFC 2119 and RFC 8174 (BCP 14). A *page* is one
Markdown file under `docs/`. A page's *kind* is one of concept, guide, or
reference.

## Boundary with doctrine

1. A page **MUST NOT** contradict doctrine.

2. A page **MUST** link a governing rule rather than restate its normative force.

3. A page **MUST NOT** use RFC 2119 keywords to impose a requirement.

4. A page **MUST NOT** duplicate a fact that can drift from its authoritative
   source.

## Structure

5. Each page **MUST** be exactly one kind.

6. `docs/` **MUST** provide a root index that routes a reader to every section.

7. Each page **MUST** declare its kind, its audience, and the doctrine it
   derives from.

## Quality

8. A page **MUST** be the shortest form that serves its purpose.

9. Every command shown **MUST** be a runnable standard-tool invocation.

10. A worked example **MUST** use a real, executed case.

11. Prose **MUST** be plain and free of marketing language.

12. Each topic **MUST** be documented on one page and linked from elsewhere.

## Coverage

13. Every fact a user needs to install, run, deploy, or extend the system
    **MUST** be reachable from `docs/`.

14. A page **MUST** lead with the common path and defer rare or exhaustive
    detail to a later section or a reference page.

## Maintenance

15. A change to user-visible behaviour **MUST** update the affected docs in the
    same change.

16. Living data **MUST** be referenced by pointer or generation rather than a
    manual snapshot.

## References

- [Process](../RULES.md) — top-level doctrine and core principles.
- [CLI](../src/RULES.md) — command transparency and reproducibility.
- [Meta rules](../meta/rules/RULES.md) — the form every `RULES.md` must take.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-06-03 | Initial version. Establishes `docs/` as the human-facing site distinct from `.agents/` (1–4), its Diátaxis structure and index (5–7), the quality bar (8–12), sufficiency and progressive disclosure (13–14), and maintenance (15–16). |
