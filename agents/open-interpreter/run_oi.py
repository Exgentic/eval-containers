#!/usr/bin/env python3
import os
import sys
from interpreter import interpreter

interpreter.llm.api_base = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
interpreter.llm.api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")
interpreter.llm.model = "openai/" + os.environ.get("EVAL_MODEL", "default")
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
        if isinstance(m, dict) and m.get("role") == "assistant" and m.get("type") == "message":
            final = m.get("content", "") or final
if not final and isinstance(messages, list) and messages:
    last = messages[-1]
    if isinstance(last, dict):
        final = last.get("content", "") or ""
print(final)
