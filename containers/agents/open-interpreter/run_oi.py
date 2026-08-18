#!/usr/bin/env python3
import os
import sys
from interpreter import interpreter

interpreter.llm.api_base = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
interpreter.llm.api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")
# EVAL_MODEL/MODEL is a bare, opaque handle that may already carry a routing
# prefix like `aws/claude-opus-4-8` or `gcp/gemini-3.5-flash-lite` (gateways/
# RULES.md: "MUST NOT split EVAL_MODEL to infer a provider or wire protocol").
# open-interpreter's Llm class has no field for litellm's custom_llm_provider
# (unlike terminus-2/harbor's LiteLLM wrapper), so a "/" in the handle reaches
# litellm.completion() unmodified and litellm tries to resolve it as a real
# provider name — "gcp"/"aws" aren't litellm provider names (those are
# "vertex_ai"/"bedrock"), so it raises BadRequestError: LLM Provider NOT
# provided. Force the wire explicitly by wrapping llm.completions (a plain
# function reference forwarding **params to litellm.completion(**params))
# to inject custom_llm_provider="openai" regardless of what the handle
# carries — same fix as #348 (terminus-2), applied via a wrapper since
# open-interpreter has no dedicated field for it.
model = os.environ.get("MODEL", "default")
interpreter.llm.model = model
_completions = interpreter.llm.completions


def _completions_via_openai_wire(**params):
    params.setdefault("custom_llm_provider", "openai")
    yield from _completions(**params)


interpreter.llm.completions = _completions_via_openai_wire
interpreter.auto_run = True
interpreter.offline = False
interpreter.disable_telemetry = True

task = os.environ.get("TASK", "")
if len(sys.argv) > 1 and not task:
    task = sys.argv[1]

messages = interpreter.chat(task, display=False, stream=False)
final = ""
if isinstance(messages, list):
    for m in messages:
        if (
            isinstance(m, dict)
            and m.get("role") == "assistant"
            and m.get("type") == "message"
        ):
            final = m.get("content", "") or final
if not final and isinstance(messages, list) and messages:
    last = messages[-1]
    if isinstance(last, dict):
        final = last.get("content", "") or ""
print(final)
