# Edge

**Status:** Draft
**Date:** August 2026

## Abstract

Every model call an agent makes crosses one component: the edge. The edge pins
the model to the handle the framework selected, forwards the call on the wire it
arrived on, records the exchange verbatim, and holds the upstream credential the
agent must never see. This document fixes what the edge must be. Cross-wire
translation and provider-specific routing remain gateway concerns.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in BCP 14, RFC 2119 and RFC 8174.

The *edge* is the component every model call crosses on its way from an agent to
an upstream provider. A *call* is one request-response exchange. The *inbound
wire* is the protocol an agent used to reach the edge. A *record* is the edge's
written account of one call.

## Principles

### Model Authority

1. **Single crossing.** Every model call an agent makes **MUST** cross the edge.

2. **Model authority.** The edge **MUST** replace the model named by the agent
   with `EVAL_MODEL` whenever `EVAL_MODEL` is set.

3. **Opaque handle.** The edge **MUST NOT** parse `EVAL_MODEL` to infer a
   provider or a wire protocol.

4. **Inbound wire preserved.** The edge **MUST** forward every call on the wire
   protocol it arrived on.

5. **No silent bridging.** The edge **MUST** answer a call that would require
   cross-wire translation with a machine-readable error.

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

### Transport

11. **Streaming preserved.** The edge **MUST** forward each response chunk to
    the agent before reading the next one.

12. **Provider-native auth.** The edge **MUST** present the upstream credential
    in the header its target wire expects.

13. **Uniform agent contract.** The edge **MUST** serve the framework's
    protocol-namespaced prefixes on port 4000.

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

- [Gateways](../gateways/RULES.md) — cross-wire translation and its declaration,
  which remain gateway concerns; rules 2b, 6, 10 and 11 there are superseded by
  this topic.
- [Verification](../verification/RULES.md) — the gates a contribution passes,
  and the fixture format the records feed.
- [Meta rules](../meta/rules/RULES.md) — rule form and no-silent-drift.
- [Contributing](../contributing/RULES.md) — contribution shape.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-08-11 | Initial version. Lifts model authority out of the gateway (superseding `gateways/RULES.md` 2b), makes call capture a property of one component rather than a per-gateway obligation (superseding 10 and 11 there), and removes the need for the path-rewriting shim (6). Translation and its declaration (8, 9) stay with gateways, which remain OPTIONAL and are needed only for cross-wire work. Status is Draft until the component lands and the verification suite covers it. |
