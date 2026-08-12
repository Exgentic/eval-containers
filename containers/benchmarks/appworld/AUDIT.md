---
benchmark: appworld
host: local Colima (amd64 via Rosetta, no QEMU)
commit: d9ef64a9
---
# Audit — appworld

`✓` verified (a check passed) · `◐` partial (holds in one surface, open in another) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build --platform linux/amd64` succeeds, no-cache, 28s of `RUN` time on top of the already-local upstream base (see Speed) |
| running | ✓ | manual bridge smoke test (below) — a real agent-shaped sequence of `POST /execute` calls against task `692c77d_2` (id `1`) |
| isolation | ✓ | agent uid 1002/gid 0 denied `/appworld`, `ground_truth/*`, and `/tasks/1/id.txt`; `GET /evaluate` 404s (no such route exists on the bridge at all); `kill -USR1` on the bridge's pid as uid 1002 → `Operation not permitted` |
| reward-hacking | ✓ | grading reuses the bridge's own live `AppWorld` object rather than trusting anything the agent reports; the reward path is written by a root-owned process the agent has no route to influence (no `/evaluate` HTTP route, no signal permission) |
| oracle | ✓ | gold = 1.0 / no-op = 0.0, confirmed live this session (see Notes) |
| traces-reviewed | ✗ | the two existing fixtures (`tests/run/replay/fixtures/appworld-{292-terminus-2,584-claude-code}.traces.jsonl`, 23/49 spans) predate this fix — they were almost certainly recorded against the old runtime, which had no data, no bridge, and no way to act, so the agent gave up immediately. They should be re-recorded against the fixed image; see Notes |
| replicate-official | ? | not attempted — would mean running the upstream `appworld` reference agent and comparing its reported score to this container's `reward.txt` |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | not audited per-benchmark; AppWorld itself needs no runtime network (data is baked at build time) |
| agent-nonroot | ✓ | agent runs as uid 1002 (this benchmark's own `/entrypoint.sh`, not the shared launcher — see Notes) |
| secrets-isolated | n/a | no LLM credentials touch this container's filesystem; the gateway holds those |
| ground-truth-isolated | ✓ | `chmod -R go-rwx $APPWORLD_ROOT` at build time; verified live as uid 1002/gid 0 — `ls`/`cat` on `/appworld` and any `ground_truth/*.json` → Permission denied |
| task-identity-hidden | ✓ | `/tasks/$EVAL_TASK_ID/id.txt` stays root:600; verified live as uid 1002 → Permission denied. The real id is scoped to the bridge's own subprocess env (`APPWORLD_TASK_ID=...`), never exported broadly |
| grading-unreachable | ✓ | the bridge exposes exactly `GET /health` and `POST /execute`; `GET /evaluate` → 404. Grading is triggered by `SIGUSR1`, and signal delivery is uid-gated by the kernel — uid 1002 sending it to the root-owned bridge pid → `Operation not permitted` (verified live) |
| resource-limited | ? | not audited here — see `compose.yaml` for the shared `compose/services.yaml` limits |

## Size

| Metric | Value |
|--------|-------|
| image | 1.06 GB (`docker images`; `docker image inspect .Size` under-reports on this multi-platform/attestation build — do not trust that field) |
| per-task multiplier | shared-env (×1) — all 732 tasks' data is baked into one image at build time |

## Speed

| Metric | Value |
|--------|-------|
| build (no-cache `RUN` time, base image already local) | 28s total — `apt-get`: 7.1s, `pip install pyarrow`: 6.0s, HF task-metadata fetch: 2.0s, `appworld install` + `download data` (~193MB): 12.1s, permission/copy steps: <1s each |
| container start → bridge healthy | 9s (`/eval-materialize-task` + cold `AppWorld()` open, confirmed 4-5s cold per appworld's own timing) |
| grade (`SIGUSR1` → `evaluate()` → `reward.txt` written) | <1s — the bridge already holds the live session in memory, no reload needed |

## Cost

| Metric | Value |
|--------|-------|
| per task | no LLM cost from the benchmark image itself (all APIs are simulated, in-process); cost is entirely the agent's own token usage |
| full suite | 732 tasks × agent token cost; no benchmark-side compute cost beyond the ~10s/task container startup above |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✓ | `ghcr.io/exgentic/benchmarks/appworld:latest` resolves (`docker manifest inspect`) |
| released label | ◐ | `LABEL eval.benchmark.released="true"` is present and fixtures do exist at the right path (rule 21a), but those fixtures predate this fix (see traces-reviewed above) — the label's *precondition* (a fixture proven against a working runtime) is currently stale |
| pull size | 1.06 GB | `docker images` |

## Notes

**The bug this audit fixes.** `output/appworld/1` showed `reward=-1, passed=false` for
every one of the 732 tasks — not because any task failed, but because there was no
AppWorld runtime for the agent to talk to at all: no task data installed, no server
running, and `/grade.sh` hardcoded `-1`. `-1` is this repo's convention for
*genuinely* externally-graded benchmarks (rule 18/20), but `world.evaluate()` is a
fully local, offline, in-process check — this was a mischaracterization, not a
real external-grading constraint.

**Root cause of a second, more subtle bug found while fixing the first.**
AppWorld's own `AppWorld.__init__` → `initialize()` → `_prepare_directories()`
(`appworld/environment.py`) does `shutil.rmtree(self.output_directory, ...)` on
every construction — opening a *second* `AppWorld(task_id=..., experiment_name=...)`
silently discards whatever the first one did, even for the same task id and
experiment name, even within the same process. An earlier version of this fix
opened a fresh `AppWorld` session per `/execute` call (to force a flush-to-disk
after every call) and per grading pass (in a separate `grade.py` process) — this
looked reasonable but is actively wrong: every solve, correct or not, always
graded 0.0, which is a worse failure mode than the original bug because it looks
like it works. The official usage pattern (confirmed in `appworld/cli.py`) is one
long-lived session for the whole task — `execute()` already persists to disk
after every call internally, so the fix is to keep exactly one `AppWorld` object
alive for the bridge's lifetime and reuse it for grading too, rather than opening
a second one.

**Grading channel: signal, not HTTP.** The agent shares the container's network
namespace with the bridge, so any TCP port bound to `127.0.0.1` is reachable to
it regardless of uid — there is no way to bind an HTTP route that only root can
reach. Signals don't have this problem: the kernel enforces that only a matching
uid (or a capable one) may `kill()` a process, so `/grade.sh` (root, run after the
agent's own process has exited) sending the root-owned bridge `SIGUSR1` is a route
the agent's uid 1002 genuinely cannot use — verified live (`Operation not
permitted`).

**Oracle re-run (this image, task `692c77d_2` / id `1`).** No-op (bridge started,
nothing executed, `/grade.sh` run immediately): `reward.txt = 0.0`, `evaluate()`
returns its full 7-assertion breakdown to `/logs/appworld/evaluation.json` for
audit. Genuine solve (real `apis.spotify.*` calls over `/execute`: paginate the
14-song library and 11 liked songs, compute the 7 unliked, update the 2 that
already had a review to rating 1, add reviews for the other 5, then
`apis.supervisor.complete_task()`): `reward.txt = 1.0`, all 7 assertions pass.
Confirmed a *separate* `/execute` call (simulating the agent's next turn) sees
the previous call's mutations — the persistence bug above is fixed, not just
worked around.

**Residuals (open):**
- **Stale replay fixtures.** The two fixtures backing the `released` label
  (`tests/run/replay/fixtures/appworld-292-terminus-2.traces.jsonl`,
  `appworld-584-claude-code.traces.jsonl`) are small (23 and 49 spans) and were
  almost certainly recorded against the pre-fix image, where the agent had
  nothing to do but give up. `tests/run/replay/test.rs`'s `assert_result_valid`
  only checks structural shape (reward in range, required fields present), not
  benchmark-specific semantics, so replaying these old fixtures against the new
  image won't fail — but they also don't demonstrate anything about the fixed
  runtime. These should be re-recorded against a real agent run before leaning
  on them as evidence of end-to-end correctness.
- **Bypasses the shared framework launcher.** This benchmark's `/entrypoint.sh`
  and `/grade.sh` are bespoke (predates this fix) rather than going through
  `/usr/local/bin/run`/`run-agent` (rule 12/22). Left as-is: migrating to the
  shared launcher is an unrelated, larger refactor and wasn't part of this fix's
  scope.
- **`egress-blocked` / `resource-limited` / `replicate-official`** — not audited
  in this pass; see `compose.yaml` for the shared network/resource config this
  benchmark inherits from `compose/services.yaml`.
