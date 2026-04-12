"""
Dock LiteLLM callback: writes LiteLLM's StandardLoggingPayload to /output/trajectory.jsonl
and maintains /output/result.json with aggregated model metadata.

The trajectory format is LiteLLM's standard — Dock only controls where it's written.
"""
import json
import os
from litellm.integrations.custom_logger import CustomLogger


class DockLogger(CustomLogger):
    def __init__(self):
        self.output_dir = os.environ.get("DOCK_OUTPUT_DIR", "/output")
        self.log_file = os.path.join(self.output_dir, "trajectory.jsonl")
        self.result_file = os.path.join(self.output_dir, "result.json")
        os.makedirs(self.output_dir, exist_ok=True)
        self.total_tokens = 0
        self.total_cost = 0.0
        self.model = ""
        self.provider = ""

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

            # Append to trajectory
            with open(self.log_file, "a") as f:
                f.write(json.dumps(payload, default=str) + "\n")

            # Update aggregated result
            self.model = payload.get("model", self.model)
            self.provider = payload.get("custom_llm_provider", self.provider)
            self.total_tokens += payload.get("total_tokens", 0) or 0
            self.total_cost += payload.get("response_cost", 0) or 0

            result = {
                "model": self.model,
                "provider": self.provider,
                "total_tokens": self.total_tokens,
                "cost_usd": round(self.total_cost, 6),
            }
            with open(self.result_file, "w") as f:
                json.dump(result, f)

        except Exception as e:
            print(f"[dock_logger] error: {e}")


dock_logger_instance = DockLogger()
