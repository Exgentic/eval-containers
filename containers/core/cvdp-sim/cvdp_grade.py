"""Run one CVDP task's own hidden harness against the agent's /code workspace.

Faithful to upstream's runner (NVlabs/cvdp_benchmark src/repository.py):
  * the harness's docker-compose.yml lists one or more services;
  * each service runs `command` in the harness home (/code/rundir, or /rundir
    for context-heavy datapoints), with `env_file` loaded, harness `src/`
    mounted at /src and the workspace at /code, and HOME / XDG_CACHE_HOME
    pointed at the harness home (docker_cmd, log_docker);
  * each service is bounded by DOCKER_TIMEOUT (600 s);
  * the task passes iff EVERY service exits 0 (obj_harness: error += result).
Upstream does this with `docker compose run`; every service image is the same
open-source simulator image this container already is, so we run the command
in-process instead. Nothing in the harness is edited.

Services whose image is upstream's synthesis image (__OSS_PNR_IMAGE__) run here
too: since v1.1.0 upstream points OSS_PNR_IMAGE at the same nvidia/cvdp-sim
image (README.md), and the only extra step their Dockerfile.synth adds is a
`pip install pytest==8.3.2`, which this image already has.

Shared by cvdp (single-turn), cvdp-agentic and cvdp-agentic-heavy: all lay the candidate files out
under /code before calling this, exactly as upstream mounts them.

argv: <task dir> <log dir>. Prints the reward (1.0 / 0.0) on stdout.
"""

import os
import re
import shutil
import subprocess
import sys

task, logs = sys.argv[1], sys.argv[2]
harness = os.path.join(task, "harness")
SERVICE_TIMEOUT = 600  # upstream DOCKER_TIMEOUT default (src/config_manager.py)


def parse_compose(text):
    """The subset of compose YAML CVDP harnesses use: services -> scalar keys + lists.

    Indentation varies across harnesses (service names at 1 or 2 spaces), so
    levels are relative: the first indent under `services:` is the service
    level, the first indent under a service is its key level. PyYAML is not in
    upstream's locked requirements, hence no YAML library.
    """
    services, cur, key, svc_ind, key_ind, in_services = (
        {},
        None,
        None,
        None,
        None,
        False,
    )
    for raw in text.splitlines():
        line = re.sub(r"\s+#.*$", "", raw.rstrip())
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip())
        body = line.strip()
        if indent == 0:
            cur, key = None, None
            in_services = body == "services:"
            continue
        if not in_services:
            continue
        if svc_ind is None:
            svc_ind = indent
        if indent == svc_ind and body.endswith(":"):
            cur, key, key_ind = services.setdefault(body[:-1].strip(), {}), None, None
        elif cur is None:
            continue
        elif (
            key_ind is not None
            and indent > key_ind
            and key
            and cur.get(key) in (">", "|")
        ):
            cur[key] = body  # block scalar (`command: >`): its single-line body
        elif body.startswith("- "):
            if key:
                cur.setdefault(key, []).append(body[2:].strip())
        elif ":" in body:
            if key_ind is None:
                key_ind = indent
            if indent != key_ind:
                continue  # nested mapping (e.g. build: dockerfile:) — not needed
            k, _, v = body.partition(":")
            key = k.strip()
            if v.strip():
                cur[key] = v.strip()
    return services


def read_env(path):
    env = {}
    if not os.path.isfile(path):
        return env
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        env[k.strip()] = v.strip()
    return env


compose = parse_compose(open(os.path.join(harness, "docker-compose.yml")).read())
os.makedirs(logs, exist_ok=True)

# Fresh, root-owned harness at /src: the agent never had it.
shutil.rmtree("/src", ignore_errors=True)
shutil.copytree(os.path.join(harness, "src"), "/src")

ok = bool(compose)
for name, svc in compose.items():
    cmd = svc.get("command")
    if not cmd:
        ok = False
        continue
    # Upstream runs `docker compose run -w <home> …`, which overrides any compose
    # working_dir, and sets HOME/XDG_CACHE_HOME to the same dir: /code/rundir for
    # regular datapoints, /rundir for context-heavy ones (repository.py
    # Repository.docker_cmd vs AgenticRepository.docker_cmd). The benchmark image
    # says which via CVDP_HARNESS_HOME.
    wd = os.environ.get("CVDP_HARNESS_HOME", "/code/rundir")
    os.makedirs(wd, exist_ok=True)
    # upstream's sim image ships /rundir/harness; ~half the harnesses point
    # pytest's cache_dir there.
    os.makedirs("/rundir/harness", exist_ok=True)
    env = dict(os.environ)
    env.update(HOME=wd, XDG_CACHE_HOME=f"{wd}/.cache")
    # env_file is a scalar or a YAML list (one non-agentic harness uses the list).
    env_files = svc.get("env_file", "./src/.env")
    for env_file in env_files if isinstance(env_files, list) else [env_files]:
        env.update(read_env(os.path.join(harness, env_file.removeprefix("./"))))
    with open(os.path.join(logs, f"{name}.log"), "w") as log:
        try:
            rc = subprocess.run(
                ["/bin/sh", "-c", cmd],
                cwd=wd,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                timeout=SERVICE_TIMEOUT,
            ).returncode
        except subprocess.TimeoutExpired:
            rc = 124
    print(f"service {name}: exit {rc}", file=sys.stderr)
    ok = ok and rc == 0

print("1.0" if ok else "0.0")
