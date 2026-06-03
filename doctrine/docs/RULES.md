# Human Documentation

**Status:** Active
**Date:** June 2026

## Abstract

`docs/` is the **human-facing** documentation site for Eval Containers — the
prose a person reads to understand the system, run an evaluation, deploy it, or
add a benchmark/agent/model. It is distinct from `doctrine/`: doctrine is the
**canonical, normative rulebook** that agents and reviewers enforce; `docs/`
**explains** the system to people and shows them how to use it. Doctrine says
what MUST be true; docs say what it is and how to do it. When the two disagree,
doctrine wins and the docs are the bug. This document defines what belongs in
`docs/`, how it is structured, and the bar every page must clear.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Boundary with doctrine

1. **Doctrine governs, docs explain.** `doctrine/` is the single source of
   truth for every rule, invariant, and design decision. `docs/` MUST NOT
   restate a rule as if it were the authority, and MUST NOT contradict
   doctrine. A page that needs to invoke a rule MUST link to the governing
   `doctrine/.../RULES.md` section rather than paraphrase its normative force.

2. **No normative language in docs.** `docs/` MUST NOT use RFC 2119 keywords
   (MUST, SHALL, …) to impose requirements — that is doctrine's job. Docs
   describe ("the runner reads `EVAL_TASK_ID`"), instruct ("run this command"),
   and explain ("because the agent must not see the proxy"). If you find
   yourself writing a requirement, it belongs in doctrine, and the doc should
   link to it.

3. **Doctrine is not duplicated.** A fact that lives in code, a chart, a
   `values.yaml`, or a `RULES.md` MUST NOT be copy-pasted into docs where it can
   silently drift. Prefer linking to the authoritative file, quoting a short
   excerpt with its source path, or generating the content. Exhaustive lists
   that must stay in sync (every CLI flag, every `EVAL_*` var, every benchmark)
   MUST cite their source so a reader and a reviewer can both find ground truth.

### Structure

4. **Three kinds of page, by purpose** (Diátaxis). Every page is exactly one of:

   | Kind | Answers | Lives in |
   |------|---------|----------|
   | **Concept** | "What is this and why?" | `docs/concepts/` |
   | **How-to guide** | "How do I accomplish task X?" | `docs/guides/` |
   | **Reference** | "What exactly are the flags / files / env vars?" | `docs/reference/` |

   A page MUST NOT mix kinds: a concept page does not become a step-by-step
   tutorial, a guide does not digress into architecture. Mixed material is split
   into one page per kind, cross-linked.

5. **`docs/README.md` is the map.** The docs root MUST open with an index that
   states the audience, lists the three sections, and links the highest-value
   entry points (run your first eval, deploy to k8s, add a benchmark). A reader
   landing cold MUST be able to find the right page in one hop.

6. **Every page declares audience and provenance.** Each page MUST open with one
   line naming its kind and intended reader (operator, contributor, …) and MUST
   link the `doctrine/` section(s) it derives from. This makes drift auditable:
   a reviewer can open the linked rule and check the page still matches.

### Content quality

7. **Concise, minimal, clean.** Docs inherit the project's simplicity and
   clean-code bars (top-level `doctrine/RULES.md` principles 7–8). A page MUST
   be the shortest version that fully serves its purpose: no filler, no
   throat-clearing, no restating what a link already says, no page that does not
   earn its place. The same discipline applied to code applies to prose —
   prefer one clear sentence over three, a table over a paragraph, a link over a
   copy. No doc outlives its purpose: one that no longer serves a reader MUST
   NOT remain in the tree, and no section MAY survive that can be cut without
   loss of meaning.

8. **Commands are real and reproducible.** Every command shown MUST be
   copy-pasteable and MUST be a plain standard-tool invocation (`docker`,
   `docker compose`, `helm`, `kubectl`, `oc`, `cargo`, `eval-containers`) that
   the reader can run as written — no pseudo-commands, no elided placeholders
   without saying what to substitute. Where the CLI is shown, the underlying
   command it stands for SHOULD be shown too, consistent with the CLI's
   transparency rule (`doctrine/src/RULES.md`).

9. **Examples run end-to-end.** A guide that walks through a task MUST use a
   concrete, runnable example (a real benchmark/agent/model), not a placeholder
   that was never executed. If an example's output is shown, it MUST be output
   that was actually produced.

10. **Plain, hype-free prose.** Docs follow the house writing standard — direct,
   concrete, no marketing adjectives, no filler. See the global `writing/RULES.md`.
   Explain the "why" once and link it; do not repeat it on every page.

11. **One concern, one home.** A given topic is documented in exactly one place
    and linked from elsewhere. If two pages would explain the same mechanism,
    one owns it and the other links. This mirrors doctrine's reuse-over-
    repetition principle and keeps drift to a single edit site.

### Maintenance

12. **Docs ship with the change.** A change that alters user-visible behavior —
    a CLI flag, a deploy command, a required per-benchmark file — MUST update the
    affected docs in the same PR. A doc that describes a removed mechanism is a
    defect, not stale background.

13. **Pointers over snapshots for living data.** Where docs reference something
    that grows or changes (the benchmark roster, the flag set), prefer a pointer
    to the authoritative source or a generated section over a hand-maintained
    snapshot. If a snapshot is unavoidable, it MUST name its source and the
    command to regenerate it.

14. **Sufficient coverage.** Everything an end user needs to know to install,
    run, deploy, or extend the system MUST be reachable from `docs/`. User-facing
    knowledge MUST NOT live only in source code, commit messages, issue threads,
    or a contributor's memory: if using a capability requires knowing a fact,
    that fact MUST have a home in `docs/` — or in a `doctrine/` page that `docs/`
    links to. A capability a user can invoke but cannot find documented is a
    defect in the docs, not an undocumented feature.

## References

- [Process](../RULES.md) — top-level doctrine and core principles
- [CLI](../src/RULES.md) — transparency / reproducibility of commands
- [Benchmarks](../benchmarks/RULES.md) — triple-mode and per-benchmark files
- Global `writing/RULES.md` — hype-free prose standard

## Changelog

| Date | Change |
|------|--------|
| 2026-06-03 | Initial version. Establishes `docs/` as the human-facing site distinct from `doctrine/`: doctrine governs, docs explain (1–3); Diátaxis structure with concepts/guides/reference and a root index (4–6); concise/minimal/clean prose carrying the project's simplicity + clean-code bars, reproducible real commands, runnable examples, hype-free prose, one-home-per-concern (7–11); docs ship with the change, pointers over snapshots (12–13). |
| 2026-06-03 | Added principle 14 (Sufficient coverage): every fact an end user needs MUST be reachable from `docs/`; user-facing knowledge MUST NOT live only in source/commits/heads. Paired with the PR templates' new docs gate so docs grow with each change. |
