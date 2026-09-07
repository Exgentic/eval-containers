# Stands in for core/otel in the output-durability test: the chart gates the
# runner on this sidecar answering GET / on :13133, and nothing else about the
# collector decides whether a result file survives its pod.
#
# The index file is not decoration — busybox httpd answers 404 for a directory
# with no index, which the chart's httpGet probe reads as unhealthy.
FROM busybox:1.37
RUN mkdir -p /www && echo ok > /www/index.html
CMD ["httpd", "-f", "-p", "13133", "-h", "/www"]
