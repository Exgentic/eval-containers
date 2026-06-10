# Project audit — the bottom line

Generated from `benchmarks/*/AUDIT.md` — do not hand-edit (regenerate with the
`audit-rollup` skill). `✓` verified · `✗` failing · `?` unchecked · `n/a` not applicable.
Safety is a rollup of the per-benchmark safety checks (`✓` only if all pass).
**Published** is computed live from the registry (`✓` the image is on
ghcr.io/exgentic/benchmarks, `✗` not), independent of whether the benchmark has
been audited. **Audited** is the date of the audit's commit (derived from it, not
stored); `⚠` means the benchmark changed after that commit, so the row is stale.

| Benchmark | Building | Running | Isolation | Oracle | Traces | Replicate | Safety | Published | Audited |
|-----------|:--------:|:-------:|:---------:|:------:|:------:|:---------:|:------:|:---------:|---------|
| `<name>` | ? | ? | ? | ? | ? | ? | ? | ✗ | `<date>` |
| `<changed-since>` | ✓ | ? | ✓ | ✓ | ? | ? | ✓ | ✓ | `<date>` ⚠ |

**Totals:** building ?/N · isolation ?/N · oracle ?/N · running ?/N · traces ?/N · replicate ?/N · safety ?/N · published ?/N
