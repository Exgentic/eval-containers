# Stands in for core/otel in the output-durability test: the chart gates the
# runner on this sidecar answering :13133, and nothing else about the collector
# matters to whether a result file survives its pod.
FROM busybox:1.37
CMD ["httpd", "-f", "-p", "13133"]
