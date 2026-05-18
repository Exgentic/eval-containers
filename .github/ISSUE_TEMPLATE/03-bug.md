---
name: Bug
about: Behavior is wrong but no rule is being violated. Something just doesn't work as intended.
title: "bug: <area> — <one-line summary>"
labels: ["bug"]
---

<!--
This template is for bugs that do NOT correspond to a rule violation.
Examples:
- The CLI crashes on a specific flag combination
- A trajectory rule mis-classifies a green run as yellow
- The live sweep double-counts cost
- cargo test fails on macOS but passes on Linux

If the bug is "code violates rule X", use the "Rule-code drift"
template instead. Drift reports carry a pointer to the rule they
violate; bug reports stand alone.

If you believe the bug means the rule itself is wrong, use the "Rule
change proposal" template instead.
-->

## Summary

<!-- One paragraph. What's broken? -->

## Steps to reproduce

```bash
# commands
```

## Expected behavior

<!-- What did you expect? Cite the relevant contract/spec/test if
you can, but no rule citation is required for bugs. -->

## Actual behavior

<!-- What did you observe? Attach stdout/stderr, stack traces,
screenshots. -->

## Environment

- **OS**: <!-- macOS 15 / Ubuntu 22.04 / ... -->
- **Arch**: <!-- arm64 / x86_64 -->
- **Runtime**: <!-- Docker Desktop X.Y / Podman X.Y with Rosetta enabled / ... -->
- **Eval Containers commit**: <!-- `git rev-parse HEAD` -->

## Scope

- [ ] Reproduces every time
- [ ] Intermittent — <!-- estimated frequency -->
- [ ] Platform-specific (fails on X, works on Y)

## Related

- [ ] Not related to any tracked issue
- [ ] Related to issue #…
