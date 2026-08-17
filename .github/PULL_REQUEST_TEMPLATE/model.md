<!--
Adding a MODEL? You don't need a PR. The model is a RUNTIME axis: set
EVAL_MODEL=<provider>/<model> + the provider key in .env — no image, no build.
There are no per-model images. See docs/guides/add-a-model.md.

Open THIS PR only to author a new GATEWAY FLAVOR (routes EVAL_MODEL at
runtime, beside bifrost/litellm/portkey): the implementation under
containers/gateways/<flavor>/ + its thin combo under containers/models/<flavor>/.
Fill every checkbox; reviewers reject on an empty evidence section.
-->

## Gateway flavor: `<name>`

<!-- One paragraph: the proxy, and why it earns a shared image — protocol
coverage, observability, or a provider the existing flavors can't reach. -->

### Contract ([.agents/gateways/RULES.md](../../.agents/gateways/RULES.md), [.agents/models/RULES.md](../../.agents/models/RULES.md))

- [ ] `containers/models/<name>/Dockerfile` is `FROM ghcr.io/exgentic/gateways/<name>:latest` and adds nothing but the config template (gateways rule 16)
- [ ] Routes `EVAL_MODEL` at runtime via a wildcard — no baked provider, model, or URL (gateways rules 1–2a; models rules 1–2)
- [ ] Labels `eval.type="model"` + `gateway.kind="<name>"` (models rule 15)
- [ ] Provider keys via provider-native env vars only — none in labels / compose / the agent (models rules 4–5)
- [ ] Logs every request + response; the agent cannot reach `/output/model/` (models rules 6–7)
- [ ] `EVAL_MODEL_MAX_BUDGET` hard cap enforced (models rule 16)
- [ ] Proxy version pinned at build time; `gateway.<name>_version` label set (models rule 12; gateways rule 14)

### Evidence: a real run

```bash
EVAL_GATEWAY_IMAGE=<name> \
  eval-containers run aime --agent claude-code --task-id 0 --local \
  --model <provider>/<model> --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty; `result.json` `cost_usd > 0`
- [ ] Swapping `--model` to another `<provider>/<model>` routes to the new provider with **no rebuild**

### Docs + changelog

- [ ] [`docs/guides/add-a-model.md`](../../docs/guides/add-a-model.md) and any affected page updated ([.agents/docs/RULES.md](../../.agents/docs/RULES.md) rule 15)
- [ ] `.agents/models/RULES.md` / `.agents/gateways/RULES.md` changelog entry if rules changed; otherwise "no RULES.md changes needed"
