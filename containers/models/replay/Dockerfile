FROM python:3.12-slim

LABEL eval.type="model"
LABEL eval.model.name="replay"
LABEL eval.model.provider="replay"

# Everything the gateway needs lives UNDER /opt/gateway, exactly like the real
# gateway flavors (bifrost/litellm) — so this image is a true drop-in gateway in
# BOTH surfaces: the compose/k8s sidecar runs the CMD, and the single-container
# standalone bundle (which `COPY`s only /opt/gateway) execs /opt/gateway/start
# in-process. That lets `--mode container` run deterministic replays too, not
# just compose/job. A `--copies` venv carries its own interpreter, so it runs
# under whatever benchmark base the standalone bundle layers onto.
RUN python3 -m venv --copies /opt/gateway/venv \
 && /opt/gateway/venv/bin/pip install --no-cache-dir flask==3.1.0
COPY server.py /opt/gateway/server.py
RUN printf '#!/bin/sh\nexec /opt/gateway/venv/bin/python /opt/gateway/server.py\n' > /opt/gateway/start \
 && printf '#!/bin/sh\nexec /opt/gateway/venv/bin/python -c "import urllib.request; urllib.request.urlopen(\\"http://localhost:4000/health\\")"\n' > /opt/gateway/health \
 && chmod +x /opt/gateway/start /opt/gateway/health

EXPOSE 4000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 \
  CMD /opt/gateway/health

CMD ["/opt/gateway/start"]
