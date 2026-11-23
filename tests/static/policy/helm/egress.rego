# tests/static/policy/helm/egress.rego — an offline benchmark's pods have no way
# out but the offline gateway (#753). The chart renders one egress NetworkPolicy
# for the pods labelled `eval-internet: "false"`; this gates what it may allow,
# and that no such pod carries a gateway sidecar, which the policy would have to
# let out with the agent. Like readiness.rego, it runs once per document.
package main

import rego.v1

np_name := object.get(input, ["metadata", "name"], "<unnamed>")

egress_rules := object.get(input, ["spec", "egress"], [])

deny contains msg if {
	input.kind == "NetworkPolicy"
	some rule in egress_rules
	count(object.get(rule, "to", [])) == 0
	msg := sprintf("NetworkPolicy/%s: an egress rule with no `to` allows every destination (#753)", [np_name])
}

deny contains msg if {
	input.kind == "NetworkPolicy"
	some rule in egress_rules
	some peer in object.get(rule, "to", [])
	peer.ipBlock
	msg := sprintf("NetworkPolicy/%s: an egress rule allows the CIDR %s; an offline pod's way out is the in-namespace gateway, not an address (#753)", [np_name, object.get(peer.ipBlock, "cidr", "?")])
}

# A peer that selects whole namespaces is the DNS rule, or it is a way out.
deny contains msg if {
	input.kind == "NetworkPolicy"
	some rule in egress_rules
	some peer in object.get(rule, "to", [])
	peer.namespaceSelector
	not dns_only(rule)
	msg := sprintf("NetworkPolicy/%s: an egress rule selects namespaces on ports other than DNS (#753)", [np_name])
}

dns_only(rule) if {
	ports := object.get(rule, "ports", [])
	count(ports) > 0
	every p in ports {
		p.port in {53, 5353}
	}
}

# An empty podSelector selects every pod in the namespace.
deny contains msg if {
	input.kind == "NetworkPolicy"
	some rule in egress_rules
	some peer in object.get(rule, "to", [])
	not peer.namespaceSelector
	count(object.get(peer, ["podSelector", "matchLabels"], {})) == 0
	count(object.get(peer, ["podSelector", "matchExpressions"], [])) == 0
	msg := sprintf("NetworkPolicy/%s: an egress rule allows every pod in the namespace (#753)", [np_name])
}

deny contains msg if {
	input.kind in {"Job", "Deployment"}
	object.get(input, ["spec", "template", "metadata", "labels", "eval-internet"], "") == "false"
	some c in object.get(input, ["spec", "template", "spec", "initContainers"], [])
	c.name == "gateway"
	msg := sprintf("%s/%s: an offline pod carries a gateway sidecar, which shares the agent's way out (#753)", [input.kind, np_name])
}
