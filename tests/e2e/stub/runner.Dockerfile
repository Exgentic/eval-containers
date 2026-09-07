# Stands in for evals/<benchmark>--<agent>: the chart invokes the runner as
# `/bin/bash -c "<runnerArgs>"`, so that path is the entire contract this test
# needs from a runner image.
#
# A symlink will not do — busybox dispatches on argv[0] and has no `bash`
# applet, so /bin/bash would answer "applet not found". A two-line wrapper does.
# (The official `bash` image is no help either: its bash is /usr/local/bin/bash.)
FROM busybox:1.37
RUN printf '#!/bin/sh\nexec /bin/sh "$@"\n' > /bin/bash && chmod +x /bin/bash
