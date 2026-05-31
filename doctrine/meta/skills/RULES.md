# Skill Rules

**Status:** Active
**Date:** May 2026

## Abstract

This document specifies what a skill is and the form every `SKILL.md` under
`doctrine/` must take. A skill is a procedure an agent can follow to produce a
result that satisfies the rules. The outcomes a skill serves are governed by
`doctrine/meta/rules/RULES.md`.

## Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in
every `SKILL.md` under `doctrine/` are to be interpreted as described in BCP 14,
RFC 2119 and RFC 8174. A *skill* is a procedure; an *agent* is the reader, human
or AI, that executes it.

## Principles

1. **Skills govern methods, not outcomes.** A skill **MUST** describe how to do
   something and **MUST NOT** introduce a new outcome requirement; outcomes
   belong in a rule.

2. **Location.** Every skill **MUST** be a directory
   `doctrine/<topic>/<skill-name>/` containing a `SKILL.md`, where
   `<skill-name>` is lowercase with words separated by hyphens.

3. **Frontmatter.** Every `SKILL.md` **MUST** begin with YAML frontmatter
   carrying a `name` and a `description`; the `description` **MUST** state what
   the skill does and when to use it.

4. **Procedure form.** The body **MUST** present ordered, imperative steps an
   agent can follow without further instruction, and **SHOULD** explain why each
   step matters.

5. **Cite the rules served.** A skill **SHOULD** cite the rules whose outcomes
   it produces, in the form `doctrine/<topic>/RULES.md:<n>`.

6. **Bundle supporting material.** A checklist, template, or script a skill
   relies on **MUST** live beside its `SKILL.md` (for example under
   `references/`, `assets/`, or `scripts/`).

7. **No surprises.** A skill **MUST NOT** contain content that surprises a
   contributor relative to its description.

## References

- RFC 2119, RFC 8174 (BCP 14).
- `doctrine/meta/rules/RULES.md` — the companion meta for rules.

## Changelog

| Date       | Change           |
|------------|------------------|
| 2026-05-31 | Initial version. |
