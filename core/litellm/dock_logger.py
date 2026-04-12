"""
Dock LiteLLM callback: writes LiteLLM's StandardLoggingPayload to /output/trajectory.jsonl

The format is LiteLLM's standard — Dock only controls where it's written.
"""
import json
import os
from litellm.integrations.custom_logger import CustomLogger


class DockLogger(CustomLogger):
    def __init__(self):
        self.log_file = os.environ.get("DOCK_LOG_FILE", "/output/trajectory.jsonl")
        os.makedirs(os.path.dirname(self.log_file), exist_ok=True)

    def log_success_event(self, kwargs, response_obj, start_time, end_time):
        self._write(kwargs)

    def log_failure_event(self, kwargs, response_obj, start_time, end_time):
        self._write(kwargs)

    async def async_log_success_event(self, kwargs, response_obj, start_time, end_time):
        self._write(kwargs)

    async def async_log_failure_event(self, kwargs, response_obj, start_time, end_time):
        self._write(kwargs)

    def _write(self, kwargs):
        try:
            payload = kwargs.get("standard_logging_object")
            if payload is None:
                return
            with open(self.log_file, "a") as f:
                f.write(json.dumps(payload, default=str) + "\n")
        except Exception as e:
            print(f"[dock_logger] error: {e}")


dock_logger_instance = DockLogger()
