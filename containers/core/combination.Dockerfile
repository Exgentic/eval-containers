# Universal LEAN eval base. Produces evals/<benchmark>--<agent>:latest.
#
# This is the EVAL, nothing more: benchmark + agent + grader + the framework
# launcher. It is what `--mode compose`, `--mode job`, and k8s run, with the
# gateway + otelcol as SIBLING containers/pods that the orchestrator starts and
# health-gates. The in-process serving glue — gateway, otelcol, process-compose,
# and the full pipeline — is NOT here; that is the single-container convenience,
# layered on top by core/standalone.Dockerfile to produce the -standalone bundle.
#
# Image naming: one image per (benchmark, agent), tag = version. The variant is a
# NAME suffix, not a tag: evals/<b>--<a> (lean base) vs evals/<b>--<a>-standalone
# (bundle). See benchmarks/RULES.md rule 24f.
#
# Build args:
#   BENCHMARK_IMAGE       — source benchmark image
#   AGENT_IMAGE           — source agent image (provides /opt/agent/)
#   AGENT_VERSION         — pinned upstream CLI version (recorded in version.json)
#   GOSU_IMAGE            — core/gosu (drop-privileges helper; used in all modes)
#
# Path layout in the lean base:
#   /opt/agent/                      COPY'd from AGENT_IMAGE
#   /run.sh                          agent launch script (image root, not /opt/agent/)
#   /usr/local/bin/gosu              from GOSU_IMAGE (drop to agent uid)
#   /usr/local/bin/run               framework launcher (runner sequence / single-image)
#   /usr/local/bin/run-agent         shared agent launcher (rule 7 allow-list, one home)
#   /usr/local/bin/write-result      final result writer
#   /usr/local/bin/materialize-task  per-task setup helper
#   /usr/local/bin/reap-sidecars     post-run sidecar reaper (k8s shareProcessNamespace)
#   /root/tasks/                     benchmark task data (mode 0700 root-only)
#   /grade.sh                        verifier (benchmark CMD)
#   /entrypoint.sh                   benchmark setup (benchmark ENTRYPOINT)
#
# Why root-owned /root: agent uid 1002 cannot traverse it (mode 0700 by
# default), so task data + verifier are unreadable. RULES.md model rule met by
# file perms alone — no Linux capabilities required.
ARG BENCHMARK_IMAGE
ARG AGENT_IMAGE
ARG AGENT_VERSION
ARG GOSU_IMAGE=ghcr.io/exgentic/core/gosu:latest

# Named stages for the build-arg base images: buildx forbids variable
# expansion in `COPY --from=` ("variable expansion is not supported for
# --from"), so pin each base to a stage here — `FROM` *does* allow the
# `${ARG}` (declared in the global scope above) — and the layers below copy
# from the stage name. buildah accepts either form; this builds on both.
FROM ${AGENT_IMAGE} AS agent
FROM ${GOSU_IMAGE}  AS gosu

FROM ${BENCHMARK_IMAGE}

ARG AGENT_VERSION

# ─── Agent layer ─────────────────────────────────────────────────────
COPY --from=agent /opt/agent/install.sh /tmp/agent-install.sh
COPY --from=agent /opt/agent/ /opt/agent/
# Agent launch script lives at the image root, not under /opt/agent/ — copy it too.
COPY --from=agent /run.sh /run.sh
RUN chmod +x /run.sh
# Reinstalling agents resolve their version from the agent image's
# /opt/agent/VERSION (written from the agent's ARG AGENT_VERSION), unless this
# build overrides it via --build-arg AGENT_VERSION. Single source of truth, so
# install and label can never disagree (RULES.md principle 9).
RUN AGENT_VERSION="${AGENT_VERSION:-$(cat /opt/agent/VERSION 2>/dev/null)}" \
      bash /tmp/agent-install.sh && rm /tmp/agent-install.sh

# ─── Drop-privileges helper ──────────────────────────────────────────
# gosu lets `run`/`run-agent` switch root → agent uid before launching the
# agent. process-compose is NOT copied here — it is a single-container-only
# orchestrator and ships only in the -standalone bundle.
COPY --from=gosu /bundle/bin/gosu /usr/local/bin/gosu

# Ensure the agent user (uid 1002) and /home/agent exist so benchmarks that
# ask the agent to write files there (e.g. AIME's answer.txt) work correctly.
RUN grep -q '^agent:' /etc/passwd || echo 'agent:x:1002:0::/home/agent:/bin/bash' >> /etc/passwd \
 && grep -q '^agent:' /etc/shadow || echo 'agent:!:19500:0:99999:7:::' >> /etc/shadow \
 && grep -q '^agent:' /etc/group  || echo 'agent:x:1002:' >> /etc/group \
 && mkdir -p /home/agent \
 && chown -R 1002:0 /home/agent \
 && chmod -R g+rwX /home/agent

# ─── Framework scripts ───────────────────────────────────────────────
COPY runner/run              /usr/local/bin/run
COPY runner/run-agent        /usr/local/bin/run-agent
COPY runner/write-result     /usr/local/bin/write-result
COPY entrypoint/eval-materialize-task /usr/local/bin/materialize-task
COPY entrypoint/reap-sidecars         /usr/local/bin/reap-sidecars

# Tighten perms. /root stays 0700 by default (root-only task data); the helpers
# are made executable here.
RUN chmod 0755 /usr/local/bin/gosu \
 && chmod +x /usr/local/bin/run \
              /usr/local/bin/run-agent \
              /usr/local/bin/write-result \
              /usr/local/bin/materialize-task \
              /usr/local/bin/reap-sidecars

# Inherited ENTRYPOINT /entrypoint.sh execs this. Override the benchmark's
# default CMD /grade.sh so the stitched image launches the pipeline (rule 12).
CMD ["/usr/local/bin/run"]
