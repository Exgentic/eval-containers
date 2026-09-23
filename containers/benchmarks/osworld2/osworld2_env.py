"""Run one OSWorld 2.0 task against a desktop container instead of a VM.

v2 states a task as a `BaseTask` subclass with its own `setup()` and
`evaluate()`, not as the JSON v1 used, and `DesktopEnv` already dispatches to
those when a task_config carries them — so the work here is loading the class
and pointing the provider at the sidecar.

  setup <task-id>   run the task's own setup (entrypoint)
  grade <task-id>   run its own evaluate, print the reward (grader)

grade must NOT reset the env: that would re-run setup over the agent's work.
"""

import importlib
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


def load_task(task_id: str):
    """The task class for `task_017` — one module, one BaseTask subclass."""
    from desktop_env.task_base import BaseTask

    module = importlib.import_module(task_id)
    for name in dir(module):
        obj = getattr(module, name)
        if isinstance(obj, type) and issubclass(obj, BaseTask) and obj is not BaseTask:
            return obj()
    raise SystemExit("%s declares no BaseTask subclass" % task_id)


def main():
    mode, task_id = sys.argv[1], sys.argv[2]
    task = load_task(task_id)
    env = de.DesktopEnv(
        provider_name="docker",  # shimmed above; the fixed set has no "container"
        path_to_vm="container",
        action_space="pyautogui",
        headless=True,
        os_type="Ubuntu",
        force_disable_vnc=True,
        force_disable_recording=True,
    )
    use_socket(env)
    if mode == "setup":
        env.reset(task_config=task)
        print("setup complete: %s" % task_id)
    elif mode == "grade":
        env._set_task_info(task)
        print("%s" % env.evaluate())
    else:
        raise SystemExit("usage: osworld2_env.py setup|grade <task_NNN>")


if __name__ == "__main__":
    main()
