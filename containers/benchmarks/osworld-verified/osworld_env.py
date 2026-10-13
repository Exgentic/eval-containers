"""Point upstream's DesktopEnv at the desktop sidecar instead of a VM.

Every upstream provider boots and snapshots a VM. The desktop here is a sibling
container that is already running, so the provider collapses to "report where
it is"; setup, postconfig and grading stay upstream's code, untouched.

  setup <task.json>   run the task's own config steps (entrypoint)
  grade <task.json>   run postconfig + the evaluator, print the reward (grader)

grade must not call reset(): that would re-run setup and wipe the agent's work.
"""

import json
import os
import socket
import sys
import time
from urllib.parse import quote, unquote

import requests
import requests_unixsocket

ROOT = "/opt/OSWorld"
sys.path.insert(0, ROOT)

from desktop_env.providers.base import Provider, VMManager  # noqa: E402
import desktop_env.desktop_env as de  # noqa: E402

# An address, as every upstream provider reports, never a name: Chrome's
# debugging port refuses a request whose Host is a name other than localhost,
# and under compose the desktop has one.
HOST = socket.gethostbyname(os.environ.get("EVAL_DESKTOP_HOST", "desktop"))
SOCKET = "http+unix://" + quote("/run/desktop/desktop.sock", safe="")
FILES = (
    "https://huggingface.co/datasets/xlangai/ubuntu_osworld_file_cache/resolve/main/"
)


class ContainerProvider(Provider):
    def start_emulator(self, path_to_vm, headless, os_type=None, *a, **k):
        return None

    def get_ip_address(self, path_to_vm):
        # ip:server:chromium:vnc:vlc — the shape the docker provider returns
        return "%s:5000:9222:5910:8080" % HOST

    def save_state(self, path_to_vm, snapshot_name):
        return None

    def revert_to_snapshot(self, path_to_vm, snapshot_name):
        return path_to_vm

    def stop_emulator(self, path_to_vm):
        return None


class ContainerManager(VMManager):
    def initialize_registry(self, **k):
        return None

    def add_vm(self, vm_path, **k):
        return None

    def delete_vm(self, vm_path, **k):
        return None

    def occupy_vm(self, vm_path, pid, **k):
        return None

    def list_free_vms(self, **k):
        return []

    def check_and_clean(self, **k):
        return None

    def get_vm_path(self, **k):
        return "container"


de.create_vm_manager_and_provider = lambda *a, **k: (
    ContainerManager(),
    ContainerProvider(None),
)


def route() -> None:
    """Answer this process's requests from where the image keeps the answer.

    The control server listens on a socket the agent cannot open, and its port
    carries only the agent's restricted face. So every desktop URL goes to the
    socket, whoever composed it — upstream's controllers or a task's own getter.
    A task's files are in the image, so their URLs are answered from disk.
    """
    requests_unixsocket.monkeypatch()
    desktop = tuple("http://%s:5000" % h for h in (HOST, "localhost", "127.0.0.1"))
    send = requests.Session.request

    def request(self, method, url=None, *a, **k):
        # requests passes `url` as a keyword, so this must accept one.
        if isinstance(url, str):
            if url.startswith(FILES):
                response = requests.Response()
                response.status_code, response.url = 200, url
                path = unquote(url[len(FILES) :].split("?")[0])
                response.raw = open("%s/files/%s" % (ROOT, path), "rb")
                return response
            for prefix in desktop:
                if url.startswith(prefix):
                    url = SOCKET + url[len(prefix) :]
                    break
        return send(self, method, url, *a, **k)

    requests.Session.request = request


def make_env():
    # provider_name is validated against a fixed set; "docker" selects the
    # ip:ports code path the provider above mimics.
    return de.DesktopEnv(
        provider_name="docker",
        path_to_vm="container",
        action_space="pyautogui",
        headless=True,
        os_type="Ubuntu",
        # Upstream's default is the working directory, which the agent reads.
        cache_dir=ROOT + "/cache",
        # reset() returns an observation nobody reads here; without this it
        # also dumps the desktop's whole accessibility tree.
        require_a11y_tree=False,
    )


def main():
    mode, task_path = sys.argv[1], sys.argv[2]
    cfg = json.load(open(task_path))
    route()
    env = make_env()
    if mode == "setup":
        env.reset(task_config=cfg)
        # reset() retries a failed setup and then carries on regardless; this
        # is the flag it raises only when the task's own steps went through.
        if cfg.get("config") and not env.is_environment_used:
            raise SystemExit("setup failed: %s" % cfg["id"])
        print("setup complete: %s" % cfg["id"])
    elif mode == "grade":
        env._set_task_info(cfg)
        # Upstream's runner gives the desktop this long to settle first.
        time.sleep(20)
        # A metric may answer True/False; grade.sh takes a number.
        print(float(env.evaluate()))
    else:
        raise SystemExit("usage: osworld_env.py setup|grade <task.json>")


if __name__ == "__main__":
    main()
