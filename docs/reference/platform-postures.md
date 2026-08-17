# Platform postures

*Reference · for operators · what an eval image demands of its execution
environment, and which platforms grant it. Derives from
[`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rules 6–9 and
13. Measurements are reproduced by `tests/run/uid/test.rs`.*

## What an eval image demands

An eval image needs **uid 0 at start plus `CAP_SETUID`/`CAP_SETGID`** — nothing
more. It uses them once, to drop the agent to uid 1002 (`gosu` in
`containers/core/runner/run-agent`), then never needs privilege again.

Measured with everything else removed
(`--cap-drop=ALL --cap-add=SETUID --cap-add=SETGID`): the drop succeeds and all
three isolation boundaries hold.

No `privileged`, Docker socket, device, `/dev/fuse`, `SYS_ADMIN`, `SYS_PTRACE`,
or sub-1024 port is required. The gateway binds 4000, otel 4318.

Beyond identity, five benchmarks need more than one container
(`containers/benchmarks/_chart/presets/`: webarena and enterpriseops-gym 8
services, visualwebarena 5, tau-bench 4, osworld 2), webarena needs a 2 GiB
`/dev/shm`, and the chart mounts memory-backed `emptyDir` for `/tmp` (1 Gi) and
`/logs` (100 Mi).

## Why the drop is load-bearing

Three boundaries are enforced by root-vs-non-root file permissions, so they exist
only if the drop happens:

| | Boundary | Mechanism |
| - | -------- | --------- |
| B1 | Gateway credentials | `/opt/gateway` root-owned `0700` (rule 8) |
| B2 | Grading artifacts | `/output` root-owned, not group-writable |
| B3 | Answers and grading code | `chmod 600 /tasks/all.jsonl` (52 benchmarks); `chmod -R 700 /tests` + `chown root:root` (16) — rules 6 and 7 |

Two consequences of the drop failing, both measured:

- **Root reads everything.** Root bypasses file permissions, so a container that
  stays at uid 0 defeats all three boundaries at once.
- **Some agents refuse root.** `claude-code` exits with
  `--dangerously-skip-permissions cannot be used with root/sudo privileges` at
  uid 0; `IS_SANDBOX=1` does not override it. `codex` is unaffected, so this is
  per-agent.

A baked `USER` is not a substitute: it was tried and reverted because
uid-assigning platforms override it, and Cloud Run has been
[observed](https://issuetracker.google.com/issues/275620724) ignoring it.

## Platform support

`✓` granted · `✗` denied. **Source** distinguishes *measured* (reproduced with
`docker run` flags) from *vendor* (documented) from *silent* (no documented
policy, so the container-runtime default applies — inferred, not promised).

| Platform | uid 0 | SETUID | Writable fs | Runs | Source |
| -------- | :---: | :----: | :---------: | :--: | ------ |
| Docker / Compose, GitHub Actions | ✓ | ✓ | ✓ | **yes** | measured |
| OpenShift + `anyuid` SCC | ✓ | ✓ | ✓ | **yes** | vendor + prior run |
| Kubernetes, no PSA | ✓ | ✓ | ✓ | **yes** | silent |
| AWS Fargate (ECS/EKS) | ✓ | ✓ | ✓ | likely | vendor (root default) |
| HF Jobs, Code Engine, SageMaker, Vertex AI, Azure Container Instances | ✓ | ? | ✓ | likely | silent |
| Modal | ✓ | ? | ✓ | ? | vendor (gVisor); uid silent |
| E2B / Daytona / Runloop | ? | ? | ? | ? | **unchecked** |
| Fargate, ECS.5 hardened | ✓ | ✓ | **✗** | **no** | measured |
| Google Cloud Run | ✓ | **✗** | ✓ | **no** | vendor |
| OpenShift `restricted` / `nonroot`, k8s PSA `restricted` | **✗** | ✗ | ✓ | see below | vendor + measured |

**Only the restrictive platforms document a uid policy.** Cloud Run, OpenShift,
and Kubernetes PSA state one, and those are the three that deny what we need.
Everywhere else the docs describe `docker run` semantics and say nothing about
uids or capabilities, so those rows record the runtime default rather than a
vendor guarantee — a hardening default could change any of them.

### Cloud Run — root, no setuid

Cloud Run's [container contract](https://docs.cloud.google.com/run/docs/container-contract)
states it "doesn't support binaries that use `setuid` flags" and recommends
testing with `--cap-drop=setuid`. Measured equivalent: the container stays at uid
0 and `gosu` fails with *operation not permitted* — so B1–B3 all fall and
`claude-code` will not start.

### Fargate ECS.5 — read-only root filesystem

The AWS Foundational Security Best Practices control ECS.5 recommends
`readonlyRootFilesystem: true`. Measured under `--read-only`: the runtime `mkdir
/output` fails with *Read-only file system*.

### Assigned-uid platforms — runs, but not cheat-resistant

OpenShift `restricted`/`nonroot` and Kubernetes PSA `restricted` assign an
arbitrary uid (always with GID 0) and drop all capabilities. Red Hat's
[image guidelines](https://docs.redhat.com/en/documentation/openshift_container_platform/4.20/html/images/creating-images)
and the Kubernetes
[Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)
`restricted` profile (`runAsNonRoot: true`, `capabilities.drop: ["ALL"]`) both
require it.

Making the writable paths GID-0 group-writable at build time is enough to let a
container **start and run** under an assigned uid. What cannot be recovered is
B3: agent and verifier then hold the **same uid**, so no file permission admits
one and excludes the other. Measured under `--user 12345:0`,
`/tasks/all.jsonl` is unreadable — the verifier cannot reach the answers it must
grade against, and loosening the mode would hand them to the agent.

A separate verifier container is the only mechanism that restores B3, and it does
not generalize: code benchmarks grade **inside** the agent's mutated workspace
(swe-bench's `/grade.sh` does `cd /testbed`, `git diff`, reinstalls the patched
package, reruns the baked tests), so there is no seam to split along.

On these platforms an eval therefore **runs but is not cheat-resistant**. The
current chart shares one `runner` container for agent and verifier, so this
applies to k8s mode as built.

## Known gap: rule 24d is unimplemented

[Rule 24d](../../.agents/benchmarks/RULES.md) requires the standalone bundle to
enforce rule 9 (no agent internet) with `iptables -m owner --uid-owner` on the
agent uid. It does not: `iptables` is absent from the image, no source file
references it, and `gosu agent getent hosts pypi.org` resolves. Five benchmark
`AUDIT.md` files record the consequence — `bigcodebench` notes a single-image
agent could fetch the public `canonical_solution`.

Implementing it as written would demand `CAP_NET_ADMIN`, which `restricted` and
PSA `restricted` both deny. K8s mode meets rule 9 by a different route (rule 8:
the runner holds no credentials, so direct egress fails auth).

## Verification status

```bash
cargo test --test uid -- --ignored
```

Every measurement here is `docker run` against a local engine. That reproduces
the uid and capability mechanism but **not** a platform's admission control — no
row has been verified against a live cluster. Open, cheapest first:

1. **HF Jobs** — `cpu-basic` is $0.01/hour; one `hf jobs run` settles a *silent*
   row empirically.
2. **OpenShift `restricted`** on the live ROKS cluster — the row the limitation
   above is asserted from.
3. **Cloud Run** — confirm or lift the one documented exclusion.
4. **E2B / Daytona / Runloop** — never checked; microVM and gVisor platforms tend
   to have capability opinions.
