# Framework-free tooling prototype (issue #114)

Gauges the feel of replacing the Rust `eval-containers-tests` crate with
standard, framework-free tooling, per issue #114. Throwaway/alongside — does
**not** delete the Rust suite or change any rule yet.

## `helm` gate — `helm.sh`

Faithful port of `tests/helm.rs` (both tests: every-benchmark render+validate,
and the `#18`/`#21` gateway-readiness ordering check).

| | Rust (`helm.rs`) | shell (`helm.sh`) |
|---|---|---|
| Lines | 185 | 60 |
| New deps | testcontainers + tokio + reqwest + bollard (compiled, unused — the test only shells to `helm`) | none (`helm`, `kubeconform`, `xargs` already required) |
| Cold run | ~46s compile + 6.9s run | 0s compile + 8.9s run |
| Result | 2 tests green, 102 benchmarks | green, 102 benchmarks, shellcheck-clean, fail-loud verified |

The helm gate shells out to `helm template` and inspects text — it touches none
of the Rust container stack, yet building it drags the whole testcontainers /
tokio / reqwest tree through the compiler. That is issue #114's thesis in one
gate: a `helm template` loop wearing a Rust integration-test crate.

Run: `tests/prototype/helm.sh`

## `check` gate — `check.bats`

Faithful port of `tests/sanity/check.rs`. Each Rust `#[test]` → one bats
`@test` (10:10), so the rule↔test catalog pairing survives.

| | Rust (`check.rs`) | bats (`check.bats`) |
|---|---|---|
| Lines | 588 | ~250 |
| Tests | 10 | 10 (same names) |
| New deps | (shares the crate's testcontainers/tokio compile) | `bats-core` (one binary; `brew`/`apt`/submodule) |
| Run | 0.04s — **after** the crate compiles (~46s cold, cached in CI) | ~22s wall / 8.8s cpu, **0 compile** (reducible with `bats --jobs`) |
| Result | 10 green | 10 green, parity verified, fail-loud verified |

The engine is plain `grep`/`find` over files in **both** — the Rust adds a
compile step and a crate, bats adds per-test process overhead. Compiled Rust
file-I/O is far faster to *execute* (0.04s); the cost is the compile tax on
every change. For a daemon-free lint that is mostly "does this file contain this
line", the bats version is the simpler standard-tools expression.

## Feel notes (honest)

- **Pure win:** `helm` (and any shell-out gate). The Rust test only ever
  shelled to `helm`/`kubeconform`; the crate it lived in cost a full
  testcontainers/tokio/reqwest compile for zero use.
- **Clean port:** the static file lints (`check`; by extension
  `dockerfile_inspection`, `task_inspection`, `compose`, `upstream_pins`).
  They are line-contains / count checks — native shell.
- **Where shell strains:** the fixture filename parse
  (`<bench>-<task>-<agent>`) needs a greedy `sed -E` to keep multi-segment
  names like `math-500` — fiddlier than the Rust. Tolerable, but the kind of
  string logic Rust expresses more clearly.
- **The hard part — now prototyped (see below):** the container/protocol-matrix
  gates. Decision #2 resolved in favor of a **compose-native oracle**.

## `gateways` (the hard part) — `gateways/` compose-native oracle

Port of the no-creds runtime slice of `tests/gateways/test.rs`: start the
portkey gateway and POST the protocol matrix. The test *is* a container —
`docker compose up --exit-code-from tester` is the whole run.

Covers `boot_portkey` (via `depends_on: condition: service_healthy`),
`portkey_anthropic_returns_501_not_implemented`, and
`portkey_genai_returns_501_not_implemented`.

| | Rust (`gateways/test.rs`, no-creds slice) | compose-native (`gateways/`) |
|---|---|---|
| Mechanism | testcontainers-rs `GenericImage` + reqwest + tokio | `compose.yaml` + a 40-line `sh` oracle |
| New deps | testcontainers + reqwest + tokio + bollard | none (tester reuses the gateway image's curl+sh) |
| Offline | yes | yes (Caddy short-circuits before any upstream) |
| Run | compile + ~1 min | ~14s, no compile |
| Verified | — | green; **fail-loud proven** — pointed at a translating flavor (bifrost) the oracle reports ✗ (bifrost attempts upstream → 500, not portkey's 501), so it distinguishes the contract |

This is the cell the issue called "the hard part to express well in shell," and
the compose-native oracle expresses it cleanly: lifecycle + ordering come from
compose (`depends_on`/`service_healthy`, the same primitives the product ships),
and the assertions are a small POSIX script. No second language, no test
framework — the most manifesto-aligned option ("just containers + compose; you
own it").

- **compose-native vs pytest+testcontainers:** compose-native wins here.
  pytest+testcontainers would swap testcontainers-rs for testcontainers-py —
  a *different* framework and a second language — to get assertions the shell
  already expresses. The credentialed-200 and OTel-emission cells (release-only
  `#[ignore]`) follow the same shape: an oracle service that POSTs with real
  creds, or one with `/output` mounted that greps `traces.jsonl` for the
  `gen_ai.*` attrs. Not prototyped (need creds), but low-risk — same pattern.

## Recommendation (all three prototypes)

1. **Worth doing — yes.** Every gate ported to a *simpler*, framework-free
   form, and the helm/gateway gates shed a testcontainers/tokio/reqwest compile
   they never used. This aligns the quality gate with the manifesto and
   `.agents/src/RULES.md`.
2. **Live/runtime tests — compose-native oracle**, not pytest+testcontainers.
3. **`cli_conformance` ↔ `cli/src/naming.rs` — resolved.** Relocated into the
   CLI crate at `cli/tests/cli_conformance.rs` (this PR): a crate's integration
   tests belong in `<crate>/tests/`, and this one guards `naming.rs`, so it
   lives with the code it tests. No cross-crate dep, no "shared data" hack;
   `cargo test -p eval-containers --test cli_conformance`. The clean split is
   now: `cli/` owns all Rust testing; the fleet tests go framework-free.
4. **Remaining unknown for a full plan:** `replay` (2507 lines — trajectory
   analysis, likely `jq`/python over fixtures).
5. The `.agents/verification/RULES.md` rule-6 change (mandates testcontainers
   today) must ship as a **separate PR** from the code (PR rule R-3).
