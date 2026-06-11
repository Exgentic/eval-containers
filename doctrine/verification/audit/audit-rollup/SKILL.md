---
name: audit-rollup
description: >-
  Generate or refresh containers/AUDIT.md — the project-level table that
  shows the bottom line of every benchmark's audit in one grid. Use it after any
  benchmark's AUDIT.md changes, or to see fleet-wide audit status at a glance. It
  scrapes each benchmarks/<name>/AUDIT.md and copies status verbatim, never
  inventing status the source files don't carry. For one benchmark's report use
  audit-benchmark.
---

# Generate the project audit table

The `containers/AUDIT.md` rollup is the one place to see, across the fleet, how far each
benchmark has actually been audited. It is generated, never hand-edited, so it
cannot drift from the per-benchmark reports it summarizes. Serves
`doctrine/verification/audit/RULES.md:9` and `:10`.

## Steps

1. **Collect.** Read every `benchmarks/<name>/AUDIT.md` — its `commit` and each
   validity and safety status. A benchmark with no `AUDIT.md` is a row of `?`; its
   absence is itself a bottom line worth showing (RULES.md:9).

2. **Roll up safety.** Reduce the four safety checks to one column: `✓` only if
   all four are `✓`, `✗` if any is `✗`, else `?`. The table shows the headline;
   the per-benchmark file holds the detail.

3. **Resolve Published live.** Ask the registry which benchmark images exist —
   one `gh api /orgs/Exgentic/packages?package_type=container` call (filter
   `benchmarks/<name>`), or `docker manifest inspect ghcr.io/exgentic/benchmarks/<name>`.
   Set **Published** `✓` if present, `✗` if not. Compute it live, like the Audited
   date — so the column is complete even for benchmarks with no `AUDIT.md` (RULES.md:9).

4. **Render the table.** From `references/project-template.md`, write one row per
   benchmark — Building, Running, Isolation, Oracle, Traces, Replicate, Safety,
   Published, Audited — copying each status verbatim from its source. The Audited
   cell is the commit's date (`git show -s --format=%cs <commit>`), not a stored
   field. Never upgrade a `?` to a `✓` (RULES.md:10); the rollup is only as audited
   as its sources.

5. **Flag stale rows.** For each benchmark, compare its `AUDIT.md` `commit` to the
   latest commit touching its *sources* — `git log -1 --format=%H --
   benchmarks/<name>/ ':(exclude)benchmarks/<name>/AUDIT.md'`. Exclude the report
   itself, or committing an `AUDIT.md` would mark its own benchmark stale. If the
   sources changed since, mark its Audited cell stale (`⚠`): a green status against
   old code is worse than none (RULES.md:11).

6. **Total each column.** Append a line counting `✓` per column (e.g. oracle
   X/N, published Y/N), so the fleet's audit and publish progress read off the bottom.

7. **Write to the root.** Write the table to `./AUDIT.md` and commit it.
   Regenerate it whenever any benchmark's `AUDIT.md` changes, so the rollup and
   its sources never disagree (RULES.md:10).
