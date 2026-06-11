FROM python:3.12-slim

LABEL eval.type="model"
LABEL eval.model.name="replay"
LABEL eval.model.provider="replay"

RUN pip install --no-cache-dir flask==3.1.0

COPY server.py /app/server.py

# Gateway-shape health probe so this image is drop-in compatible with the
# `gateway` service contract (compose/k8s healthcheck calls /opt/gateway/health).
RUN mkdir -p /opt/gateway \
 && printf '#!/bin/sh\nexec python3 -c "import urllib.request; urllib.request.urlopen(\\"http://localhost:4000/health\\")"\n' > /opt/gateway/health \
 && chmod +x /opt/gateway/health

EXPOSE 4000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 \
  CMD /opt/gateway/health

CMD ["python3", "/app/server.py"]
