# tests/static/policy/helm/jobname.rego — the Job name is an interface, so assert
# on it rather than trusting whichever template happens to produce it (#426).
#
# `eval.jobName` (_helpers.tpl) is the one definition; this gate holds whatever
# renders the name to the properties that definition exists to guarantee, so a
# second copy of the rule cannot quietly reappear and drift away from the
# launchers that mirror it.
#
# #426 was exactly that drift: the rule lived twice, and the copy nobody called
# had lost its `toString`, so an unset axis reached printf as nil and rendered
# the literal `%!s(<nil>)` — which, since `%` cannot start a YAML token, did not
# even parse. Hence the template-artifact rule below.
#
# Charset is deliberately not asserted here yet: per-task task ids may carry `_`,
# which RFC 1123 forbids, and sanitizing them is #372/#373's contract to add. A
# rule that passes only because this matrix renders no such id would assert
# nothing.
package main

import rego.v1

job_name := name if {
	input.kind == "Job"
	name := object.get(input, ["metadata", "name"], "")
}

# A name is required: k8s rejects the object, but the render succeeds, so the
# failure would otherwise surface at apply time as a cluster problem.
deny contains msg if {
	job_name == ""
	msg := "Job: rendered with an empty metadata.name"
}

# Unrendered template output in the name — `%!s(<nil>)`, `%!d(...)`, `<nil>` —
# means an axis reached printf as the wrong type. Go templates emit this
# silently; only the YAML parser (or the cluster) ever complains.
deny contains msg if {
	some artifact in ["%!", "<nil>"]
	contains(job_name, artifact)
	msg := sprintf(
		"Job/%s: name carries unrendered template output (%s) — an axis reached printf untyped (#426)",
		[job_name, artifact],
	)
}

# RFC 1123 bounds an object name at 63 characters. eval.jobName truncates to 54
# and appends an 8-char sha1 precisely so a long per-task id stays inside it; a
# name past the bound means that path was bypassed.
deny contains msg if {
	count(job_name) > 63
	msg := sprintf(
		"Job/%s: name is %d characters — RFC 1123 bounds it at 63, which eval.jobName truncates to hold",
		[job_name, count(job_name)],
	)
}
