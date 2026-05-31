---
name: Rule change proposal
about: A rule is wrong, stale, or counterproductive. Propose a specific change to the normative document.
title: "rfc: <RULES.md file>#<rule id> — <one-line summary>"
labels: ["rfc", "rules"]
---

<!--
This template is for the case where the RULE should change, not the
code. Rule change is a higher bar than drift: you're asking everyone
to re-read the contract, not asking one file to be updated. Include
rationale, impact analysis, and migration path.

If the rule is just stale wording that doesn't match current code
(e.g. renamed env var), that's drift in the docs — use the
"Rule-code drift" template instead. Rule change is for semantic
changes to what the rule demands.
-->

## Which rule

- **Document**: <!-- e.g. `doctrine/agents/RULES.md` -->
- **Rule number / section**: <!-- e.g. rule 13 "Runtime version override" -->
- **Current text** (pasted verbatim):

> …

## Proposed new text

> …

## Rationale

<!-- Why does the current rule fail? Concrete examples, real costs,
specific benchmarks or agents that are forced into unnatural shapes
by the rule. Not abstract preferences. -->

## Impact

- [ ] Code changes required: <!-- which files/tests need updating -->
- [ ] Other rules invalidated: <!-- which RULES.md principles need coordinated updates -->
- [ ] Mechanical check changes: <!-- which rules in tests/sanity/ need to be added / removed / rewritten -->
- [ ] Breaking for contributors: <!-- does this invalidate existing PRs in flight? existing fixtures? existing external integrations? -->

## Migration path

<!-- If accepted, how does this roll out? All-at-once edit? Phased
deprecation with a grace period? New contributors get the new rule
immediately while existing artifacts are grandfathered? -->

## Alternatives considered

<!-- What other ways could this problem be addressed? Why is the
proposal above preferred? -->

## Related drift

<!-- If this proposal is motivated by an audit that turned up
systemic drift, link to the audit file. -->
