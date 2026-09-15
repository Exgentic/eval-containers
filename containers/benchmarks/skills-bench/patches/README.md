# Upstream environment patches

`build.sh` applies `<task>.patch` to the pinned skillsbench checkout before it
builds that task's `environment/Dockerfile`. One patch per task, nothing else.

A patch belongs here only when the upstream environment is broken in a way that
is not about the task: a floating dependency that has since moved. We do not
fork the tasks, and we never touch what the task actually tests.

The patch is a diff against the `REF` pinned in `build.sh`. If that ref moves
and a patch no longer applies, `git apply` fails and the build fails — which is
the point: someone has to look at whether upstream fixed it and the patch can go.

| task | why |
|---|---|
| `earthquake-phase-association` | pip resolves the newest `setuptools`; 82 removed `pkg_resources`, which `seisbench==0.10.2` imports on import. Constrained to `<81`. |
| `seismic-phase-picking` | same. |
