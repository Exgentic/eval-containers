# Edge

**Status:** Draft
**Date:** August 2026

## Abstract

Every model call an agent makes crosses one component: the edge. It pins the
model to the handle the framework selected, forwards the call on the wire it
arrived on, and records the exchange verbatim — the request as the agent sent
it, tool schemas included. This document fixes what the edge must be. Wire
translation, provider routing and trace emission remain gateway concerns; the
edge sits in front of a gateway, it does not replace one.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in BCP 14, RFC 2119 and RFC 8174.

The *edge* is the component every model call crosses on its way from an agent to
an upstream provider. A *call* is one request-response exchange. The *inbound
wire* is the protocol an agent used to reach the edge. A *record* is the edge's
written account of one call. A *total* is the edge's running sum, over the calls
it has recorded, of the token counts their responses report.

## Principles

### Model Authority

1. **Single crossing.** Every model call an agent makes **MUST** cross the edge.

2. **Model authority.** The edge **MUST** replace the model named by the agent
   with `EVAL_MODEL` whenever `EVAL_MODEL` is set.

3. **Opaque handle.** The edge **MUST NOT** parse `EVAL_MODEL` to infer a
   provider or a wire protocol.

4. **Inbound wire preserved.** The edge **MUST** forward every call on the wire
   protocol it arrived on.

5. **No silent bridging.** The edge **MUST** refuse to start when it is
   configured to translate wires, since a gateway does that.

### Capture

6. **Verbatim capture.** The edge **MUST** record every call as the agent sent
   it, before the model is replaced.

7. **Whole exchange.** Each record **MUST** contain the request body, the
   response body, and the response status.

8. **Chunk timing.** Each record **MUST** carry the arrival time of every
   response chunk.

9. **Credential exclusion.** A record **MUST NOT** contain the upstream
   credential.

10. **Capture is unconditional.** Recording **MUST NOT** be disabled by default.

10a. **Running total.** The edge **MUST** maintain a total beside the record.

10b. **Unread usage is not zero.** The edge **MUST** count a call whose token
     counts it cannot read as unread in the total.

### Transport

11. **Streaming preserved.** The edge **MUST** forward each response chunk to
    the agent before reading the next one.

12. **Provider-native auth.** The edge **MUST** present the upstream credential
    in the header its target wire expects.

13. **Uniform agent contract.** The edge **MUST** serve the protocol-namespaced
    prefixes fixed by [gateways 5](../gateways/RULES.md), on a port distinct
    from the gateway's.

### Form

14. **Credential isolation.** An agent **MUST NOT** be able to read the upstream
    credential the edge holds.

15. **No prerequisites.** The edge **MUST** be a single statically linked
    executable that runs with no runtime dependency.

16. **Shell-free readiness.** The edge **MUST** report its readiness through its
    own executable.

17. **Environment configuration.** The edge **MUST** take its configuration from
    environment variables only.

## References

- [Project rules](../RULES.md) — principle 5, independent observation: the
  general requirement this topic refines into one component.
- [Gateways](../gateways/RULES.md) — translation, routing and OTel emission,
  which remain gateway concerns; only model authority (2b) moves here.
- [Verification](../verification/RULES.md) — the gates a contribution passes,
  and the fixture format the records feed.
- [Meta rules](../meta/rules/RULES.md) — rule form and no-silent-drift.
- [Contributing](../contributing/RULES.md) — contribution shape.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-09-08 | Added 10a (running total) and 10b (unread usage is not zero), with *total* in Terminology. A reader that wants a task's token usage had to read the records, and a record is the whole conversation repeated once per call — the dashboard paid 917s of saturated CPU deriving it for one sweep of the history (Exgentic/dashboard#109). The edge already holds each response when the call completes; every other reader has to decompress the file to get back to it. Additive: 6-9 are unchanged, the record stays verbatim and whole, and a run recorded before this has no total, which readers already handle. 10b exists because a usage shape the edge cannot read would otherwise be indistinguishable from a call that used no tokens (#500). |
| 2026-08-18 | Scoped to what the component actually is: an addition in front of the gateway, not a replacement for it. Capture rules (6-10) stand on their own; gateways keep translation, routing and OTel emission, and only model authority moves. An earlier draft superseded those too, which turned a small fix into a fleet-wide migration. |
| 2026-08-12 | Pre-merge review, while Draft. Rule 5 now requires refusing to start rather than answering each call with an error, matching how the gateway `start` scripts reject bad env — and what the implementation does. Rule 13 binds to gateways 5 and 7 instead of restating the namespace and port, which mirrored a rule that already has a home (meta 4). |
| 2026-08-11 | Initial version. Lifts model authority out of the gateway (superseding `gateways/RULES.md` 2b), makes call capture a property of one component rather than a per-gateway obligation (superseding 10 and 11 there), and removes the need for the path-rewriting shim (6). Translation and its declaration (8, 9) stay with gateways, which remain OPTIONAL and are needed only for cross-wire work. Status is Draft until the component lands and the verification suite covers it. |
