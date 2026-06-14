FROM python:3.12-slim

LABEL eval.type="core"
LABEL eval.core.component="llm-bridge"
LABEL eval.core.description="Fake OpenAI API that queues requests for a real agent"

RUN pip install --no-cache-dir flask
COPY bridge.py /app/bridge.py
CMD ["python", "/app/bridge.py"]
