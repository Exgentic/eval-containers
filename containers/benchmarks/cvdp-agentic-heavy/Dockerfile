# syntax=docker/dockerfile:1
# CVDP — Comprehensive Verilog Design Problems (NVIDIA, arXiv:2506.14074),
# AGENTIC-HEAVY variant: agentic tasks set inside full open-source hardware
# repositories (CVA6, cv32e40p, PCIe, Ethernet, Caliptra, …), the "heavy context"
# split. Sibling of `cvdp-agentic` (self-contained tasks) and `cvdp` (single-turn).
# Upstream: github.com/NVlabs/cvdp_benchmark. Dataset: nvidia/cvdp-benchmark-dataset.
#
# Scope: the 75 tasks of v1.1.0 `agentic_heavy_code_generation_no_commercial`
# (all `hard`; 56 debugging cid016, 7 module reuse, 6 spec→RTL, 6 modification)
# over 36 repositories. Each task's context is a sanitized commit in a git
# bundle; the 36 bundles (~1.3 GB) are baked root-only and the entrypoint checks
# the task's commit out at run time, as upstream's runner does.
#
# Grading is CVDP's own harness, unmodified (core/cvdp-sim/cvdp_grade.py), with
# upstream's context-heavy harness home: `-w /rundir`, HOME=/rundir.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives the rows, the bundles and the label.
ARG BENCHMARK_VERSION=5b807d945f6a99aa645f7e43a64a2115e281b4bf

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
ARG CVDP_SPLIT=cvdp_v1.1.0_agentic_heavy_code_generation_no_commercial
COPY materialize.py /materialize.py
RUN python3 - <<'PY'
import os, subprocess, urllib.request
rev = os.environ["BENCHMARK_VERSION"]
base = f"https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset/resolve/{rev}"
urllib.request.urlretrieve(f"{base}/{os.environ['CVDP_SPLIT']}.jsonl", "/tmp/split.jsonl")
subprocess.run(["python3", "/materialize.py", "/tmp/split.jsonl", "/tasks"], check=True)
os.makedirs("/tasks/bundles", exist_ok=True)
for repo in open("/tasks/repos.txt").read().split():
    url = f"{base}/cvdp_v1.1.0_agentic_heavy_code_generation_public/{repo}.bundle"
    for attempt in range(5):
        try:
            urllib.request.urlretrieve(url, f"/tasks/bundles/{repo}.bundle"); break
        except Exception as e:  # noqa: BLE001
            print(f"fetch {repo} attempt {attempt + 1} failed: {e}")
    else:
        raise SystemExit(f"could not fetch {url}")
PY

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}cvdp-sim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="cvdp-agentic-heavy"
LABEL eval.benchmark.description="CVDP agentic heavy - hard Verilog debug/design tasks inside full open-source hardware repositories, graded by CVDP's own cocotb harnesses (open-source tools)"
LABEL eval.benchmark.tasks="75"
LABEL eval.benchmark.env="shared-env"
ARG EVAL_INTERNET=false
LABEL eval.benchmark.internet="${EVAL_INTERNET}"
ENV EVAL_INTERNET=${EVAL_INTERNET}
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/NVlabs/cvdp_benchmark"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2506.14074"
ENV EVAL_BENCHMARK=cvdp-agentic-heavy
# Upstream's context-heavy harness home (AgenticRepository.docker_cmd: -w /rundir).
ENV CVDP_HARNESS_HOME=/rundir

COPY --from=data /tasks /tasks
RUN chmod -R 600 /tasks && find /tasks -type d -exec chmod 700 {} + && chmod 711 /tasks

COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
