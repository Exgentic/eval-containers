---
name: Rule-code drift
about: A rule says X, the code does Y. The rule is correct; the code needs to catch up.
title: "drift: <RULES.md file>#<rule id> — <one-line summary>"
labels: ["drift"]
---

<!--
This template is for the case where a normative document says one
thing and the implementation says another. If you think the RULE
should change instead of the code, use the "Rule change proposal"
template. If you think both the rule and the code agree but behavior
is still wrong, use the "Bug" template.

Drift reports are the highest-value signal this repo gets — they
keep the rules graph honest. Include evidence, not opinion.
-->

## Which rule

- **Document**: <!-- e.g. `benchmarks/RULES.md` -->
- **Rule number / section**: <!-- e.g. rule 22 "Shared components" -->
- **Rule text** (pasted verbatim):

> …

## Evidence of drift

<!-- Link to the specific file:line(s) where the drift is visible.
Show both what the rule demands and what the code does. -->

- **File**: <!-- e.g. `benchmarks/appworld/Dockerfile:40-60` -->
- **What the rule demands**:
- **What the code does**:

<details><summary>Minimal reproduction (optional but encouraged)</summary>

```
<command + output>
```

</details>

## Expected fix direction

- [ ] Update the code to satisfy the rule (this is the default for drift)
- [ ] Add a mechanical check so this drift can't recur silently
- [ ] Document as known-broken (only if the drift is environmentally unfixable — platform, upstream dep, etc. — and needs to live in `tests/build/known-broken.md` or similar)

## Scope

- [ ] Single artifact (one benchmark, one agent, one model)
- [ ] Fleet-wide (many artifacts violate the same rule — bulk fix via sub-agents)

## Related drift audits

<!-- If this drift was found during a walked audit, link to the
audit file (tests/audit-*.md). Otherwise delete this section. -->
