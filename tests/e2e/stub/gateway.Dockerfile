# Stands in for models/<gateway>: the chart gates the runner on this sidecar's
# /opt/gateway/health, and the test makes no model calls.
FROM busybox:1.37
RUN mkdir -p /opt/gateway && printf '#!/bin/sh\nexit 0\n' > /opt/gateway/health \
    && chmod +x /opt/gateway/health
CMD ["sleep", "infinity"]
