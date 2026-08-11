# The edge — every model call crosses it (.agents/edge/RULES.md).
#
# Static, stdlib-only Go: the binary has no runtime dependency (rule 15), so the
# runtime stage is `scratch` and the same file can instead be COPY'd into the
# single-container standalone bundle. It lives at /opt/gateway/start to keep the
# framework's existing entrypoint contract, and answers `start health` itself so
# no shell is needed for the probe (rule 16).
FROM golang:1.26.5-alpine AS build
WORKDIR /src
COPY go.mod main.go ./
RUN CGO_ENABLED=0 go vet ./... \
 && CGO_ENABLED=0 go build -ldflags="-w -s" -trimpath -o /opt/gateway/start .

FROM scratch
LABEL eval.type="core"
LABEL eval.core.component="edge"
LABEL eval.core.description="Pins the model to EVAL_MODEL and records every LLM call verbatim"

COPY --from=build /opt/gateway/start /opt/gateway/start

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
