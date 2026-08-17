# Use (or add) a model

*Guide · for everyone · the canonical rules are [`.agents/models/RULES.md`](../../.agents/models/RULES.md).*

The model is a **runtime** axis: one shared gateway image routes any model, and
**using a model never requires a build**. There are no per-model images — the
only build case is authoring a new gateway *flavor*.

## Any model, zero build

The shared gateway routes whatever `EVAL_MODEL=<provider>/<model>` you set to
that provider, so any model works with no new image:

```bash
echo "OPENAI_API_KEY=sk-..." > .env
eval-containers run aime --task-id 0 --agent codex --model openai/gpt-5.4
# or --model anthropic/claude-sonnet-4-5, gemini/gemini-2.5-pro, openai/azure/<deployment>, …
```

`EVAL_MODEL` must be `<provider>/<model>` form — the gateway errors on a
bare name or an empty value (no silent default). Pick the proxy flavor with
`EVAL_GATEWAY_IMAGE` (default `bifrost`; `litellm` and `portkey` also ship) —
the flavor changes *how* requests are proxied, never *which* model runs.

The gateway holds the real provider key; the runner only ever sees the proxy —
see [Isolation & gateways](../concepts/isolation-and-gateways.md).

## Custom gateway config (advanced, non-default)

If your team needs a shared, custom-configured gateway — a fixed corporate
endpoint, custom cost rates, an alias table — that is a **downstream artifact
you own**, not a framework image (models rule 1a). Two equivalent styles, both
selected through the standard seams:

```bash
# Style A — mount your template on the published gateway image
docker run -e EVAL_MODEL=<provider>/<model> -e <provider-creds> \
  -v ./config.yaml.template:/opt/gateway/config.yaml.template \
  ghcr.io/exgentic/gateways/litellm:latest
```

```dockerfile
# Style B — bake it in YOUR registry: template-only over a published flavor
FROM ghcr.io/exgentic/gateways/litellm:latest
COPY config.yaml.template /opt/gateway/config.yaml.template
```

Publish style B under your own registry (e.g. `ghcr.io/<you>/models/<name>`)
and select it with `EVAL_REGISTRY=ghcr.io/<you> EVAL_GATEWAY_IMAGE=<name>`
(compose) or `--set gatewayImageRef=<full-ref>` (k8s). The image must stay
template-only — same routing as the bare gateway with the template mounted.
Custom images never live in this repo or its registry: the framework's default
stays one shared gateway per flavor.

## Add a gateway flavor (the only build case)

You build + publish an image only to **author a new gateway flavor** beside
`bifrost`/`litellm`/`portkey` — never for a model. Then:

1. **Read the rules** — [`.agents/gateways/RULES.md`](../../.agents/gateways/RULES.md)
   (provider-agnostic, `EVAL_MODEL` selected at runtime) and
   [`.agents/models/RULES.md`](../../.agents/models/RULES.md)
   (rule 1: no per-model images; rules 4–7: key isolation + tamper-proof logging).
2. **Ship the pair** — the implementation at `containers/gateways/<flavor>/` and
   its thin combo at `containers/models/<flavor>/` (gateways rules 16–17).
3. **Open the PR** with the
   [model PR template](../../.github/PULL_REQUEST_TEMPLATE/model.md).
