"""Grade one SWE-fficiency task with upstream's own harness; README.md.

usage: grade.py <work dir> <agent patch>   (writes <work dir>/result.json)

Upstream evaluates each prediction with `run_instance` in a fresh container of the
task's image. This container already is that image, so the Docker calls are served
locally and each "fresh container" is the pristine /testbed and conda tree that
the entrypoint saved before the agent ran.
"""

import io
import json
import pickle
import shlex
import subprocess
import sys
import tarfile
import threading
from collections import namedtuple
from pathlib import Path
from types import SimpleNamespace

# Only this process: the scripts the harness runs must see the testbed env alone.
sys.path.insert(0, "/tests/deps")

from swefficiency.harness import run_validation  # noqa: E402
from swefficiency.report import evaluate_instance  # noqa: E402

work, patch = Path(sys.argv[1]), Path(sys.argv[2]).read_text()
instance = json.loads(Path("/tasks/0/config.json").read_text())
spec = pickle.loads(Path("/tests/spec.pkl").read_bytes())
ExecResult = namedtuple("ExecResult", "exit_code output")


def argv(cmd):
    return shlex.split(cmd) if isinstance(cmd, str) else cmd


class Api:
    def __init__(self):
        self.procs = {}

    def exec_create(self, container_id, cmd, **_):
        self.procs[len(self.procs)] = cmd
        return {"Id": len(self.procs) - 1}

    def exec_start(self, exec_id, stream=False, **_):
        proc = subprocess.Popen(
            argv(self.procs[exec_id]),
            cwd="/testbed",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.procs[exec_id] = proc
        yield from iter(lambda: proc.stdout.read(65536), b"")
        proc.wait()

    def exec_inspect(self, exec_id):
        return {"Pid": self.procs[exec_id].pid}


class Container:
    id = "local"
    client = SimpleNamespace(api=Api())

    def start(self):
        subprocess.run(
            "rm -rf /testbed /opt/miniconda3/envs/testbed"
            " && cp -a /tests/pristine/testbed /testbed"
            " && cp -a /tests/pristine/env /opt/miniconda3/envs/testbed",
            shell=True,
            check=True,
        )

    def exec_run(self, cmd, workdir="/testbed", detach=False, **_):
        if detach:
            subprocess.Popen(argv(cmd), cwd=workdir)
            return ExecResult(0, b"")
        r = subprocess.run(
            argv(cmd), cwd=workdir, stdout=subprocess.PIPE, stderr=subprocess.STDOUT
        )
        return ExecResult(r.returncode, r.stdout)

    def put_archive(self, path, data):
        tarfile.open(fileobj=io.BytesIO(data)).extractall(path)
        return True

    def remove(self, **_):
        pass

    def get_archive(self, path):
        buf = io.BytesIO()
        with tarfile.open(fileobj=buf, mode="w") as tar:
            tar.add(path, arcname=Path(path).name)
        return iter([buf.getvalue()]), {}


run_validation.create_container_from_image = lambda *args, **kwargs: Container()
run_validation.cleanup_container = lambda *args, **kwargs: None
client = SimpleNamespace(images=SimpleNamespace(pull=lambda *a, **k: None))


def evaluate(name, model_patch, **flags):
    pred = {
        "instance_id": instance["instance_id"],
        "model_name_or_path": name,
        "model_patch": model_patch,
    }
    # run_instance reads its worker index from the thread's name.
    thread = threading.Thread(
        target=run_validation.run_instance,
        args=(spec, pred, False, False, client, "grade"),
        kwargs={
            "timeout": 7200,
            "run_perf": True,
            "use_dockerhub_images": True,
            **flags,
        },
        name="worker_0",
    )
    thread.start()
    thread.join()
    return work / "logs" / "run_evaluation" / "grade" / name


# The expert's speedup is measured here, beside the agent's, as upstream's gold run
# measures it; its correctness is not part of the score, so its tests are not run.
gold = evaluate("gold", instance["patch"])
agent = evaluate("agent", patch, run_correctness=True)
verdict = evaluate_instance(instance, gold, agent)
ratio = max(verdict["human_speedup_ratio"], 0.001)
result = {"reward": 1 - 0.001 / ratio, **verdict}
(work / "result.json").write_text(json.dumps(result, indent=2, default=float))
print(json.dumps(result, default=float))
