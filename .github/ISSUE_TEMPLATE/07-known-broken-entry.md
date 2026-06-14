---
name: Known-broken entry
about: Document a benchmark / agent / fixture that is currently broken under a specific condition.
title: "known-broken: <name> — <one-line condition>"
labels: ["known-broken"]
---

<!--
This template is for documenting something that IS currently broken,
when the fix is not immediate and we want the break to be visible
and tracked. A known-broken entry is NOT a drift report and NOT a
rule change — it's a housekeeping record of a gap.

Known-broken manifests live in:
- tests/build/known-broken.md       (build-time failures)
- tests/run/replay/fixtures/broken.json (recorded trajectory failures)
- tests/run/live/known-broken.md        (live-run failures, future)

The issue you open with this template feeds one of the manifests
above.
-->

## Which artifact

- **Type**: benchmark / agent / model / fixture
- **Name**: `<name>`
- **Condition**: <!-- e.g. "arm64 laptop without HF_TOKEN", "gpt-5.4 + codex", "release v1.2 only" -->

## What's broken

<!-- Paste the actual failure: exit code, stack trace, rule
violation, refusal message, whatever the symptom is. -->

## Root cause

- [ ] Platform: runs on CI but not on my arm64/podman/etc. (document the CI runner that works)
- [ ] Upstream gated: needs credentials or a signup form
- [ ] Upstream gone: URL 404, package yanked, base image missing
- [ ] Code drift: the current code violates a rule (please ALSO open a drift issue)
- [ ] Rule drift: the current rule is wrong for this artifact (please ALSO open an RFC)
- [ ] Unknown

## Proposed manifest entry

<!-- Paste the row you want added to the known-broken manifest,
in the format of the target file. Example for tests/build/known-broken.md: -->

```
| `<name>` | `<upstream>` | `<gate>` | <HF_TOKEN fixes? yes/no/maybe> |
```

## Workaround for operators

<!-- What can someone running the sweep locally do about it right
now? "Skip this benchmark" / "Accept 32 GB disk usage" / "Use
x86_64 CI" / "Wait for upstream to accept my license request". -->

## When to revisit

<!-- What would need to be true for this to become fixable?
"Upstream releases arm64 build" / "We get an enterprise HF_TOKEN"
/ "LiteLLM v2.x supports the Responses API cost field". -->
