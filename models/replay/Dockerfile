FROM python:3.12-slim

LABEL dock.type="model"
LABEL dock.model.name="replay"
LABEL dock.model.provider="replay"

RUN pip install --no-cache-dir flask

COPY server.py /app/server.py

EXPOSE 4000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 \
  CMD python3 -c "import urllib.request; urllib.request.urlopen('http://localhost:4000/health')"

CMD ["python3", "/app/server.py"]
