# Stands in for models/<gateway>: the chart gates the runner on this sidecar's
# /opt/gateway/health, and it answers the one endpoint the mock agent asks for.
#
# It is not a model. It exists to prove the wiring an agent depends on — that
# OPENAI_API_BASE points somewhere the runner container can actually reach, in a
# pod where the gateway is a sidecar rather than a service. Recorded traffic and
# real traces are the replay model's job (models/replay, tests/run/replay).
FROM busybox:1.37
RUN mkdir -p /opt/gateway /www/v1 \
 && printf '#!/bin/sh\nexit 0\n' > /opt/gateway/health && chmod +x /opt/gateway/health \
 && printf '{"object":"list","data":[{"id":"stub","object":"model"}]}\n' > /www/v1/models
CMD ["httpd", "-f", "-p", "4000", "-h", "/www"]
