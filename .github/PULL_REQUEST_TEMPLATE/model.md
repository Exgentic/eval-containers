<!--
Adding a MODEL? You almost certainly don't need a PR. The model is a RUNTIME
axis: set EVAL_MODEL=<provider>/<model> (any LiteLLM-supported model) + the
provider key in .env — no image, no build. See docs/guides/add-a-model.md.

Open THIS PR to author a MODEL IMAGE under containers/models/<name>/ — either:
  - a PINNED per-model artifact (bakes one model + its config; a shared,
    versioned reference teams run against via EVAL_GATEWAY_IMAGE=<name>), or
  - a new GENERIC backend (routes EVAL_MODEL at runtime, beside bifrost/litellm/portkey).
Fill every checkbox; reviewers reject on an empty evidence section.
-->

## Model image: `<name>`

- [ ] **Kind**: pinned per-model (bakes `<provider>/<model>`) **/** generic backend (routes `EVAL_MODEL`)

<!-- One paragraph: the model or proxy, and why it earns a shared image — team
reproducibility, custom cost/endpoint/params, protocol coverage, or a provider
the generic backends can't reach. -->

### Contract ([.agents/models/RULES.md](../../.agents/models/RULES.md))

- [ ] `FROM ghcr.io/exgentic/gateways/<proxy>:latest` or `core/litellm:latest` (inherits the eval-logger + budget wrapper)
- [ ] Routing matches the kind — a generic backend routes `EVAL_MODEL` via a wildcard (rules 1–2); a pinned image bakes one `<provider>/<model>`
- [ ] Labels `eval.type="model"` (+ `gateway.kind` / `eval.model.*` per rule 15)
- [ ] Provider keys via `os.environ/<PROVIDER>_API_KEY` only — none in labels / compose / the agent (rules 4–5)
- [ ] Logs every request + response; the agent cannot reach `/output/model/` (rules 6–7)
- [ ] `EVAL_MODEL_MAX_BUDGET` hard cap enforced (rule 16)
- [ ] LiteLLM/proxy version pinned as a reproducible default; `EVAL_LITELLM_VERSION` honored (rules 12–13)

### Evidence: a real run

```bash
# generic backend: EVAL_GATEWAY_IMAGE=<name> + --model <provider>/<model>
# pinned per-model: EVAL_GATEWAY_IMAGE=<name>   (model is baked; no --model)
EVAL_GATEWAY_IMAGE=<name> \
  eval-containers run aime --agent claude-code --task-id 0 --local --max-budget 1
```

- [ ] `output/aime/0/model/trajectory.jsonl` non-empty; `result.json` `cost_usd > 0`
- [ ] (generic) swapping `--model` to another `<provider>/<model>` routes to the new provider with **no rebuild**

### Docs + changelog

- [ ] [`docs/guides/add-a-model.md`](../../docs/guides/add-a-model.md) and any affected page updated ([.agents/docs/RULES.md](../../.agents/docs/RULES.md) rule 15)
- [ ] `.agents/models/RULES.md` changelog entry if rules changed; otherwise "no RULES.md changes needed"
