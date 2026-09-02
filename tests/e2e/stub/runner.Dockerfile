# Stands in for evals/<benchmark>--<agent>: the chart invokes the runner as
# `/bin/bash -c "<runnerArgs>"`, so that path is the entire contract this test
# needs from a runner image. (The official `bash` image does not satisfy it —
# its bash is at /usr/local/bin/bash.)
FROM busybox:1.37
RUN ln -s /bin/sh /bin/bash
