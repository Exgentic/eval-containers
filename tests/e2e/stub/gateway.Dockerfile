# Stands in for models/<gateway>: the chart gates the runner on this sidecar's
# /opt/gateway/health, and it answers the one endpoint the mock agent asks for.
#
# The health script really probes the port rather than exiting 0: the chart holds
# the runner behind this, so a health check that passes before httpd is listening
# lets the agent start too early and find nothing there — which is exactly the
# bug a real gateway's health contract exists to prevent.
#
# It reports `usage` the way a provider does, which is not decoration: the edge
# counts a call from what the response says, so a stub that reports nothing makes
# every spend cap unreachable in a real run.
#
# It is not a model. It exists to prove the wiring an agent depends on — that
# OPENAI_API_BASE points somewhere the runner container can actually reach, in a
# pod where the gateway is a sidecar rather than a service. Recorded traffic and
# real traces are the replay model's job (models/replay, tests/run/replay).
FROM busybox:1.37
RUN mkdir -p /opt/gateway /www/v1 \
 && printf '#!/bin/sh\nwget -qO- http://127.0.0.1:4000/v1/models >/dev/null\n' > /opt/gateway/health \
 && chmod +x /opt/gateway/health \
 && printf '{"object":"list","data":[{"id":"stub","object":"model"}],"usage":{"prompt_tokens":11,"completion_tokens":3,"total_tokens":14}}\n' > /www/v1/models
CMD ["httpd", "-f", "-p", "4000", "-h", "/www"]
