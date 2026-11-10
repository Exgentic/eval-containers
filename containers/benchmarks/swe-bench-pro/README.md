# swe-bench-pro

SWE-bench Pro **V2**: 642 validated, professional-grade software engineering tasks across 11 repositories.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 642 (HARD-51 subset: upstream `v2/hard51_ids.txt`) |
| Environment | per-task |
| Internet required | false (agent); the verifier keeps network, see below |
| Released | no |
| Upstream | [scaleapi/SWE-bench_Pro-os `v2/`](https://github.com/scaleapi/SWE-bench_Pro-os/tree/v2.0.0/v2) |
| Leaderboard | [Scale — SWE-bench Pro (public, V2)](https://labs.scale.com/leaderboard/swe_bench_pro_public_v2) |
| Paper | [paper](https://arxiv.org/abs/2509.16941) |
| Dataset revision | tag `v2.0.0` = `66f92766bba642462d4bbe5479e83f91f9211862` |

## Version

On 2026-09-22 Scale replaced the original public split (731 tasks, ~30% of
which an external audit found broken) with V2: 642 repaired tasks, each a
self-contained Harbor task with its own verifier and image. This benchmark
tracks V2. **Scores from before this change are v1 scores and are not
comparable** (see AUDIT.md for how the old build silently drifted between the two).

Repos (task count): ansible 82, openlibrary 82, flipt 78, qutebrowser 73,
teleport 68, vuls 58, protonmail/webclients 55, element-web 52, navidrome 52,
NodeBB 38, tutanota 4. `tasks.txt` lists all 642 ids.

## What the agent sees

Upstream's own `instruction.md`: the PR description, its requirements, and the
new interfaces the tests expect. The repository is at `/app` (upstream notes a
few tasks may use `/testbed`; the scripts handle both).

## How it's graded

Upstream's locked protocol, run in place (`grade.sh`):

1. **Capture** the agent's diff against the base commit as `model.patch`
   (committed and uncommitted work alike).
2. **Restore** the tree to the base commit and drop files the agent created.
3. **Replay** `model.patch` the way upstream's `patch_replay` agent does, then run
   upstream's `tests/test.sh` unchanged. Reward 1 iff every `fail_to_pass` and
   `pass_to_pass` test passes.

Upstream runs step 3 in a fresh container. Running it in place is the same
deviation `deepswe` and `swe-bench` document.

## Budget and resources

50 minutes per task (upstream `[agent].timeout_sec = 3000`, the protocol's
own budget), 1 CPU, 4 GiB, 10 GiB disk — identical on all 642 tasks.

## Leaderboard comparability

The official numbers use Scale's locked agent wrappers (Claude Code, Codex,
mini-swe-agent) with WebFetch/WebSearch disabled and no turn or cost cap. A run
here is comparable when it uses the same agent, the same budget, and leaves
those tools off. The private (276) and held-out (858) splits are operator-run
and not part of V2.

## Files

- `build.sh` — resolves the task's base image from its `task.toml` and builds the overlay
- `Dockerfile` — the overlay: instruction, root-only verifier, grader, entrypoint
- `grade.sh` — capture, restore, replay, then upstream's `test.sh`
- `solution.sh` — oracle: fetches upstream's gold patch and `solve.sh` at the pinned tag
- `tasks.txt` — the 642 instance ids
- `compose.yaml` — compose file for `eval-containers run swe-bench-pro`
