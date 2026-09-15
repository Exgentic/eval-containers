# Publish on demand

*Guide · for contributors and their agents · derives from [`.agents/delivery/RULES.md`](../../.agents/delivery/RULES.md) rules 5, 16, 18–20.*

`main` publishes `:latest` once a night. To get an image out before then — the
benchmark you just merged, one agent's combos, a single task — run the
**Release the fleet** workflow yourself: the Actions tab → *Release the fleet*
→ *Run workflow*, or from a terminal:

```bash
gh workflow run release-images.yml --ref main -f only="benchmark-gsm8k agent-claude-code" -f combo_agents=claude-code -f include_per_task=false
```

A dispatch starts immediately in its own lane — it never waits for the
nightly or for another dispatch. Every run publishes only images whose inputs
changed since they were last published, under their hash tag first and
`latest` after ([delivery rules 16, 18–20](../../.agents/delivery/RULES.md)),
so a dispatch is safe to repeat, "nothing built" means everything was already
current, and two runs that race on one image resolve to the first published
hash tag.

## Choosing the scope

| Input | Scope | Blank |
|---|---|---|
| `only` | space-separated leaf targets — the first column of `containers/scripts/fleet-hash.sh graph` | the whole fleet |
| `combo_agents` | agents whose `evals/<benchmark>--<agent>` combos to build, over the benchmarks in scope | no combos |
| `tasks` | task ids of a per-task benchmark named in `only` (`terminal-bench`, `skills-bench`, `deepswe`, `swe-bench`, `hwe-bench`) | every task; with `only` alone, one task per benchmark as a smoke |
| `include_per_task` | build per-task images at all | on |
| `include_standalone` | also build the `-standalone` single-container bundles | on |
| `dry_run` | print what would build; push nothing | off |
| `tag` | a `vX.Y.Z` to publish — a versioned release, not a preview ([rule 5](../../.agents/delivery/RULES.md)) | `latest` |

```bash
# one benchmark, no combos
gh workflow run release-images.yml --ref main -f only=benchmark-gsm8k -f combo_agents= -f include_per_task=false
# one agent's combos on one benchmark
gh workflow run release-images.yml --ref main -f only="benchmark-gsm8k agent-openhands" -f combo_agents=openhands -f include_per_task=false
# two swe-bench tasks, with their claude-code combos
gh workflow run release-images.yml --ref main -f only=benchmark-swe-bench -f tasks="astropy__astropy-12907 astropy__astropy-13033" -f combo_agents=claude-code
# preview what a scope would build
gh workflow run release-images.yml --ref main -f only=benchmark-gsm8k -f dry_run=true
gh run watch "$(gh run list --workflow=release-images.yml --limit 1 --json databaseId --jq '.[0].databaseId')"
```

## Finding what you published

The run summary (the `report` job) lists the leaf images in scope with their
build time and size; each job's log says `fresh` (already published at its
hash), `carried forward`, or shows the build. Every image is reachable as
`:latest` and as its hash tag:

```bash
containers/scripts/fleet-status.sh hash ghcr.io/exgentic/benchmarks/gsm8k:latest
```

Pin a run to it with `EVAL_BENCHMARK_TAG=<hash>` (`EVAL_AGENT_TAG`,
`EVAL_GATEWAY_TAG` likewise) — `latest` moves tonight, a hash tag never does.

## Dispatch from `main`

Dispatch after your change is on `main`. A dispatch on a branch publishes the
branch's images under the same `:latest` names the nightly uses, so it can put
unmerged bits behind `latest` until the next night; a branch-only preview
under hash tags alone is a planned follow-up.
