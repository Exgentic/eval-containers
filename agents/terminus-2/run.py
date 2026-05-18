#!/usr/bin/env python3
"""Run Terminus-2 agent on $TASK, print final answer to stdout.

Terminus-2 has no standalone CLI. It is an out-of-container orchestrator
that normally connects to a sandboxed environment via Harbor's BaseEnvironment.
This wrapper creates a local environment shim so it can run inside this
container for Eval Containers evaluation.
"""
import asyncio
import os
import subprocess
import sys
import tempfile
from pathlib import Path


class LocalEnvironmentShim:
    def __init__(self, trial_dir: Path):
        self.trial_paths = type("TrialPaths", (), {
            "agent_dir": trial_dir / "agent",
            "trial_dir": trial_dir,
        })()
        self.default_user = None
        (trial_dir / "agent").mkdir(parents=True, exist_ok=True)

    async def exec(self, command, cwd=None, env=None, timeout_sec=None, user=None):
        merged_env = {**os.environ, **(env or {})}
        try:
            result = subprocess.run(
                command, shell=True, cwd=cwd, env=merged_env,
                capture_output=True, text=True,
                timeout=timeout_sec or 300,
            )
            return type("ExecResult", (), {
                "stdout": result.stdout,
                "stderr": result.stderr,
                "return_code": result.returncode,
            })()
        except subprocess.TimeoutExpired:
            return type("ExecResult", (), {
                "stdout": "",
                "stderr": "Command timed out",
                "return_code": 124,
            })()

    async def start(self, force_build=False): pass
    async def stop(self, delete=False): pass
    async def attach(self): pass
    def is_dir(self, path, user=None): return Path(path).is_dir()
    def is_file(self, path, user=None): return Path(path).is_file()
    async def upload_file(self, source_path, target_path): pass
    async def upload_dir(self, source_dir, target_dir): pass
    async def download_file(self, source_path, target_path):
        import shutil
        shutil.copy2(source_path, target_path)
    async def download_dir(self, source_dir, target_dir):
        import shutil
        shutil.copytree(source_dir, target_dir, dirs_exist_ok=True)


async def main():
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    model = os.environ.get("EVAL_MODEL", os.environ.get("MODEL", "openai/gpt-4o"))
    api_base = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
    api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")
    os.environ["OPENAI_API_KEY"] = api_key

    from harbor.agents.terminus_2 import Terminus2
    from harbor.models.agent.context import AgentContext

    logs_dir = Path(tempfile.mkdtemp(prefix="terminus2-"))
    trial_dir = Path(tempfile.mkdtemp(prefix="terminus2-trial-"))

    agent = Terminus2(logs_dir=logs_dir, model_name=model, api_base=api_base, temperature=0.7)
    env = LocalEnvironmentShim(trial_dir)
    context = AgentContext()

    await agent.setup(env)
    await agent.run(instruction=task, environment=env, context=context)

    answer = ""
    if hasattr(agent, "_trajectory_steps") and agent._trajectory_steps:
        for step in reversed(agent._trajectory_steps):
            if hasattr(step, "source") and step.source == "assistant":
                if hasattr(step, "message") and step.message:
                    answer = str(step.message); break
                elif hasattr(step, "content") and step.content:
                    answer = str(step.content); break
    if not answer and context.metadata:
        answer = str(context.metadata)
    if not answer:
        answer = "[Terminus-2 completed but produced no extractable answer]"
    print(answer)


if __name__ == "__main__":
    asyncio.run(main())
