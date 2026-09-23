"""Point upstream's DesktopEnv at a desktop container instead of a VM.

Every upstream provider boots and snapshots a VM. The desktop here is an
ordinary sibling container that is already running, so the provider collapses
to "report where it is" and no-ops; setup, postconfig and grading stay
upstream's code, untouched.

Two entry points, deliberately separate processes:
  setup <task.json>   run the task's own config steps (entrypoint)
  grade <task.json>   run postconfig + the evaluator, print the reward (grader)

grade must NOT call reset(): that would re-run setup and wipe the agent's work.
It populates the task info the same way reset() does, then evaluates.
"""

import json
import os
import sys

sys.path.insert(0, "/opt/OSWorld-V2")

from desktop_env.providers.base import Provider, VMManager  # noqa: E402
import desktop_env.desktop_env as de  # noqa: E402

DESKTOP_HOST = os.environ.get("DESKTOP_HOST", "desktop")
SERVER_PORT = os.environ.get("DESKTOP_PORT", "5000")


class ContainerProvider(Provider):
    def start_emulator(self, path_to_vm, headless, os_type=None, *a, **k):
        return None

    def get_ip_address(self, path_to_vm):
        # ip:server:chromium:vnc:vlc — the shape the docker provider returns
        return "%s:%s:9222:5910:8080" % (DESKTOP_HOST, SERVER_PORT)

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


# Setup and grading talk to the real control server; the agent must not. The
# server listens on a root-only unix socket, so point the controllers at it —
# `http_server` is the single place either controller builds its URL from.
def use_socket(env) -> None:
    """Send this process's desktop traffic over the root-only socket.

    Patching the controllers is not enough: a task's own setup builds its own
    `http://localhost:5000/...` and would hit the restricted face the agent
    uses, so rewrite at the transport instead — every desktop URL this process
    emits, whoever composed it.
    """
    sock = os.environ.get("DESKTOP_SOCKET")
    if not sock:
        return
    from urllib.parse import quote

    import requests
    import requests_unixsocket

    requests_unixsocket.monkeypatch()
    sock_url = "http+unix://%s" % quote(sock, safe="")
    env.controller.http_server = sock_url
    env.setup_controller.http_server = sock_url

    port = os.environ.get("DESKTOP_PORT", "5000")
    prefixes = tuple(
        "http://%s:%s" % (h, port)
        for h in (os.environ.get("DESKTOP_HOST", "desktop"), "localhost", "127.0.0.1")
    )
    original = requests.Session.request

    def over_socket(self, method, url=None, *a, **k):
        # requests passes `url` as a keyword, so this must accept one.
        if isinstance(url, str):
            for p in prefixes:
                if url.startswith(p):
                    url = sock_url + url[len(p) :]
                    break
        return original(self, method, url, *a, **k)

    requests.Session.request = over_socket


def make_env():
    # provider_name is validated against a fixed set; "docker" selects the
    # ip:ports code path the shim above mimics.
    return de.DesktopEnv(
        provider_name="docker",
        path_to_vm="container",
        action_space="pyautogui",
        headless=True,
        os_type="Ubuntu",
        force_disable_vnc=True,
        force_disable_recording=True,
    )


def main():
    mode, task_path = sys.argv[1], sys.argv[2]
    cfg = json.load(open(task_path))
    env = make_env()
    use_socket(env)
    if mode == "setup":
        env.reset(task_config=cfg)
        print("setup complete: %s" % cfg["id"])
    elif mode == "grade":
        env._set_task_info(cfg)
        print("%s" % env.evaluate())
    else:
        raise SystemExit("usage: osworld_env.py setup|grade <task.json>")


if __name__ == "__main__":
    main()
