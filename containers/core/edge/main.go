// The edge: the one component every model call crosses (.agents/edge/RULES.md).
//
// Recording (default): pin the model to EVAL_MODEL (rule 2), forward on the
// wire the call arrived on (rule 4), stream the response back unbuffered
// (rule 11), and write the exchange as the agent sent it (rules 6-8). The
// injected upstream credential never enters a record (rule 9).
//
// One static binary with no runtime dependency (rule 15): the same file is a
// scratch image and a process inside every eval image. Stdlib apart from
// klauspost/compress, which is pure Go — it links statically and costs 256 KB on
// a 6.4 MB binary, for a record that compresses ~138x instead of gzip's ~4x.
package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/klauspost/compress/zstd"
)

// One recorded call. Headers exclude every credential-bearing one, so rule 9
// holds by construction rather than by redaction. The request is stored once,
// verbatim; `Model` is the only thing the edge changed on the way out.
type call struct {
	Path      string            `json:"path"`
	Wire      string            `json:"wire"`
	StartUnix float64           `json:"start_unix"`
	Headers   map[string]string `json:"headers"`
	Request   string            `json:"request"`
	Model     string            `json:"model,omitempty"` // what the model was pinned to
	Status    int               `json:"status"`
	RespHead  map[string]string `json:"resp_headers"`
	Response  string            `json:"response"`
	TotalMs   float64           `json:"total_ms"`
	Chunks    [][2]float64      `json:"chunks"` // [ms since request start, bytes]
	Truncated bool              `json:"truncated,omitempty"`
	Retries   int               `json:"retries,omitempty"`
}

var (
	mu sync.Mutex
	// Compression is chosen by the caller, through the name it asks for: an OUT
	// ending .zst is written through ONE encoder held open for the run. Each
	// request repeats the whole conversation, so nearly all of a record is the
	// previous one — a compressor that sees across records shrinks a real
	// trajectory ~138x, where one that restarts per record manages ~7x. Held
	// open, therefore, rather than reopened per call (rule 8 is about what is
	// recorded, not how it is framed). Default stays uncompressed, so nothing
	// that reads this file today has to change until it asks for the .zst name.
	recFile *os.File
	recZstd *zstd.Encoder
	recPath string // what recFile was opened for; reopen if OUT is repointed
	out     = envOr("OUT", "/output/model/calls.jsonl")
	// The running total (rules 10a, 10b). Beside the record, and deliberately
	// not inside it: a reader that wants the number should not have to read the
	// records, which is the whole point — each one repeats the conversation, so
	// summing a task's usage from them costs the trajectory over again.
	// Only the override is fixed here; the default follows OUT at write time,
	// the way recPath does, so repointing OUT moves both together.
	usageOverride = os.Getenv("USAGE_OUT")
	totals        usage
	// 4100, not 4000: a gateway owns 4000, and in k8s it shares this pod's
	// network namespace (edge rule 13).
	listen = envOr("LISTEN", ":4100")
	// EVAL_MODEL is the in-framework name; EDGE_MODEL is the same knob for
	// standalone use outside eval-containers (rule 3: still never parsed).
	model  = envOr("EVAL_MODEL", os.Getenv("EDGE_MODEL"))
	base   = strings.TrimSuffix(os.Getenv("OPENAI_API_BASE"), "/")
	apiKey = os.Getenv("OPENAI_API_KEY")

	// A provider serves its own native paths; a gateway serves the framework's
	// protocol-namespaced ones (gateways rule 5). Declared, never sniffed.
	upstreamIsGateway = os.Getenv("EDGE_UPSTREAM") == "gateway"

	// No client timeout: an agent turn legitimately runs for minutes.
	client = &http.Client{}

	// Gemini names the model in the URL rather than the body.
	geminiModel = regexp.MustCompile(`/models/[^:/]+(:|$)`)

	// Credentials never reach a record (rule 9); the agent's are placeholders anyway.
	secretHeader = map[string]bool{"authorization": true, "x-api-key": true, "x-goog-api-key": true}
	// A proxy must not forward these (RFC 7230 6.1).
	hopByHop = map[string]bool{
		"connection": true, "keep-alive": true, "proxy-authenticate": true,
		"proxy-authorization": true, "te": true, "trailer": true,
		"transfer-encoding": true, "upgrade": true,
	}

	// Bound how much of one exchange is written to the record; the response is
	// streamed, so this caps the file, not the footprint.
	maxRecord = envInt("EDGE_MAX_RECORD_BYTES", 8<<20)
	// Bound the request instead of trusting it: pinning the model means parsing
	// the body, so it is held whole. Past this the call is refused rather than
	// forwarded unpinned, which would break model authority (rule 2).
	maxRequest = envInt("EDGE_MAX_REQUEST_BYTES", 64<<20)
	// Transport failures are retried only before any byte reaches the agent.
	maxRetries = envInt("EDGE_MAX_RETRIES", 2)
)

func envOr(k, d string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return d
}

func envInt(k string, d int) int {
	if v, err := strconv.Atoi(os.Getenv(k)); err == nil && v >= 0 {
		return v
	}
	return d
}

func msSince(t time.Time) float64 { return float64(time.Since(t).Microseconds()) / 1000 }

// wireFor maps an inbound path onto its protocol namespace, returning the
// wire and the path with the namespace stripped.
func wireFor(path string) (string, string) {
	// The namespace matches with or without a trailing path: an SDK pointed at
	// .../anthropic probes the bare prefix, and answering that on the wrong
	// wire would send the wrong auth header.
	for _, ns := range []struct{ prefix, wire string }{
		{"/anthropic", "anthropic"},
		{"/genai", "gemini"},
		{"/openai", "openai"},
	} {
		if path == ns.prefix {
			return ns.wire, "/"
		}
		if strings.HasPrefix(path, ns.prefix+"/") {
			return ns.wire, strings.TrimPrefix(path, ns.prefix)
		}
	}
	return "openai", path
}

// upstreamPathFor keeps the protocol namespace when a gateway is behind the
// edge and strips it when a provider is, since only the gateway serves it.
func upstreamPathFor(inbound, stripped string) string {
	if upstreamIsGateway {
		return inbound
	}
	return stripped
}

// pin replaces the model the agent named with EVAL_MODEL. The field is
// top-level JSON on every body-carrying wire, so no per-wire knowledge is
// needed — and the handle is never parsed (rule 3).
func pin(body []byte, to string) []byte {
	if to == "" || len(body) == 0 {
		return body
	}
	var m map[string]any
	if err := json.Unmarshal(body, &m); err != nil {
		return body // not JSON we understand: forward untouched
	}
	m["model"] = to
	pinned, err := json.Marshal(m)
	if err != nil {
		return body
	}
	return pinned
}

// pinGemini rewrites the model named in a Gemini URL. The handle goes in
// verbatim, slashes included (rule 3).
func pinGemini(path, to string) string {
	if to == "" {
		return path
	}
	return geminiModel.ReplaceAllString(path, "/models/"+to+"$1")
}

func clip(b []byte) (string, bool) {
	if len(b) > maxRecord {
		return string(b[:maxRecord]), true
	}
	return string(b), false
}

// The running total beside the record (rule 10a). `Unread` is what rule 10b is
// for: a response whose token counts the edge cannot find is counted here, not
// folded into the totals as a zero — a reader can then tell a task that spent
// nothing from one whose usage this edge did not understand.
type usage struct {
	Calls  int   `json:"calls"`
	Input  int64 `json:"input"`
	Output int64 `json:"output"`
	Unread int   `json:"unread"`
}

// Where a response reports what a call cost. Providers agree on the shape — an
// object of counts — and disagree on every name in it, so the names are listed
// and the object is found by walking rather than by wire. That keeps rule 4's
// promise intact: the edge still does not need to know which wire it is on.
var (
	usageObjects = []string{"usage", "usageMetadata"}
	inputNames   = []string{"prompt_tokens", "input_tokens", "promptTokenCount"}
	outputNames  = []string{"completion_tokens", "output_tokens", "candidatesTokenCount"}
)

// callUsage reads one response's token counts, and reports whether it found any.
//
// The maximum across the body, not the sum, because a streamed response reports
// a *running* total: Anthropic sends input once in `message_start` and a growing
// output in each `message_delta`, and OpenAI sends the finished figure in a
// final chunk. Maximum is the one rule that reads all of those correctly, and
// re-reading a non-streamed body under it changes nothing.
func callUsage(body string) (in, out int64, found bool) {
	for _, frame := range jsonFrames(body) {
		var v any
		if json.Unmarshal([]byte(frame), &v) != nil {
			continue
		}
		i, o, f := findUsage(v)
		in, out, found = max64(in, i), max64(out, o), found || f
	}
	return in, out, found
}

// findUsage looks for the counts wherever a response nests them. Anthropic
// reports the first ones under `message` rather than at the top, so a top-level
// lookup reads a streamed call as having no input at all; walking costs nothing
// on a body already parsed and does not have to be revisited per wire.
func findUsage(v any) (in, out int64, found bool) {
	switch t := v.(type) {
	case map[string]any:
		for _, name := range usageObjects {
			if u, ok := t[name].(map[string]any); ok {
				if n, ok := number(u, inputNames); ok {
					in, found = max64(in, n), true
				}
				if n, ok := number(u, outputNames); ok {
					out, found = max64(out, n), true
				}
			}
		}
		for _, child := range t {
			i, o, f := findUsage(child)
			in, out, found = max64(in, i), max64(out, o), found || f
		}
	case []any:
		for _, child := range t {
			i, o, f := findUsage(child)
			in, out, found = max64(in, i), max64(out, o), found || f
		}
	}
	return in, out, found
}

// jsonFrames is every JSON document in a response body: the body itself, or —
// when it is an event stream — each `data:` payload. Reported without deciding
// which it is, since a wire is not what tells them apart.
func jsonFrames(body string) []string {
	if !strings.Contains(body, "data:") {
		return []string{body}
	}
	var frames []string
	for _, line := range strings.Split(body, "\n") {
		line = strings.TrimSpace(line)
		if !strings.HasPrefix(line, "data:") {
			continue
		}
		if payload := strings.TrimSpace(strings.TrimPrefix(line, "data:")); payload != "" && payload != "[DONE]" {
			frames = append(frames, payload)
		}
	}
	return frames
}

func number(m map[string]any, names []string) (int64, bool) {
	for _, n := range names {
		if v, ok := m[n].(float64); ok {
			return int64(v), true
		}
	}
	return 0, false
}

func max64(a, b int64) int64 {
	if b > a {
		return b
	}
	return a
}

// writeTotals rewrites the total. Whole and renamed into place, because a reader
// polls this file while calls are still landing and half a JSON object is not a
// smaller answer, it is an unreadable one. Failure is logged and dropped: the
// total is derived from the record, so losing it costs the shortcut, never the
// evidence.
// usagePath is where the total goes: USAGE_OUT when set, else beside whatever
// OUT currently names.
func usagePath() string {
	if usageOverride != "" {
		return usageOverride
	}
	return filepath.Join(filepath.Dir(out), "usage.json")
}

func writeTotals() {
	path := usagePath()
	tmp := path + ".tmp"
	b, err := json.Marshal(totals)
	if err == nil {
		err = os.WriteFile(tmp, append(b, '\n'), 0o644)
	}
	if err == nil {
		err = os.Rename(tmp, path)
	}
	if err != nil {
		log.Println("usage:", err)
		os.Remove(tmp)
	}
}

func record(c call) {
	mu.Lock()
	defer mu.Unlock()
	// The record lives beside the other model-service output, in a directory the
	// agent can read but not write (runner/run leaves /output/model root-owned).
	if dir := filepath.Dir(out); dir != "." {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			log.Println("record:", err)
			return
		}
	}
	if recFile == nil || recPath != out {
		if recZstd != nil {
			recZstd.Close()
			recZstd = nil
		}
		if recFile != nil {
			recFile.Close()
			recFile = nil
		}
		f, err := os.OpenFile(out, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			log.Println("record:", err)
			return
		}
		recFile, recPath = f, out
		if strings.HasSuffix(out, ".zst") {
			enc, err := zstd.NewWriter(f,
				zstd.WithEncoderLevel(zstd.SpeedBetterCompression))
			if err != nil {
				// Falling through here would write plain JSON under a .zst
				// name — a record nothing downstream can read, and no error to
				// say so. Refuse the file instead, and try again next record.
				log.Println("record:", err)
				f.Close()
				recFile, recPath = nil, ""
				return
			}
			recZstd = enc
		}
	}
	var w io.Writer = recFile
	if recZstd != nil {
		w = recZstd
	}
	if err := json.NewEncoder(w).Encode(c); err != nil {
		log.Println("record:", err)
		return
	}
	// There is no graceful shutdown — the pod SIGKILLs this process — so the
	// frame is never closed. Flushing after every record is what makes the file
	// readable anyway: a reader gets every record written so far.
	if recZstd != nil {
		if err := recZstd.Flush(); err != nil {
			log.Println("record:", err)
		}
	}
	// Under the same lock as the record it summarises, so the total can never
	// describe a set of calls different from the one on disk.
	in, out, found := callUsage(c.Response)
	totals.Calls++
	if found {
		totals.Input += in
		totals.Output += out
	} else {
		totals.Unread++
	}
	writeTotals()
}

func handle(w http.ResponseWriter, r *http.Request) {
	start := time.Now()
	wire, path := wireFor(r.URL.Path)

	agentBody, err := io.ReadAll(io.LimitReader(r.Body, int64(maxRequest)+1))
	if err == nil && len(agentBody) > maxRequest {
		http.Error(w, `{"error":{"type":"request_too_large","message":"body exceeds EDGE_MAX_REQUEST_BYTES; the edge must parse it to pin the model"}}`,
			http.StatusRequestEntityTooLarge)
		c := call{
			Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
			Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
			Status: http.StatusRequestEntityTooLarge, Truncated: true, TotalMs: msSince(start),
		}
		record(c)
		return
	}

	upstreamBody, upstreamPath := agentBody, upstreamPathFor(r.URL.Path, path)
	if wire == "gemini" {
		upstreamPath = pinGemini(upstreamPath, model)
	} else {
		upstreamBody = pin(agentBody, model)
	}

	c := call{
		Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
		Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
	}
	c.Request, c.Truncated = clip(agentBody)

	resp, retries, err := forward(r, upstreamPath, wire, upstreamBody)
	c.Retries = retries
	if err != nil {
		http.Error(w, `{"error":{"type":"upstream_unreachable","message":"`+err.Error()+`"}}`, http.StatusBadGateway)
		c.Status, c.TotalMs = http.StatusBadGateway, msSince(start)
		record(c)
		return
	}
	defer resp.Body.Close()

	for k, vs := range resp.Header {
		if hopByHop[strings.ToLower(k)] {
			continue
		}
		for _, v := range vs {
			w.Header().Add(k, v)
		}
		c.RespHead[k] = vs[0]
	}
	w.WriteHeader(resp.StatusCode)
	c.Status = resp.StatusCode

	var body bytes.Buffer
	buf := make([]byte, 32*1024)
	flusher, _ := w.(http.Flusher)
	for {
		n, rerr := resp.Body.Read(buf)
		if n > 0 {
			chunk := buf[:n]
			if body.Len() < maxRecord {
				body.Write(chunk)
			} else {
				c.Truncated = true
			}
			c.Chunks = append(c.Chunks, [2]float64{msSince(start), float64(n)})
			if _, werr := w.Write(chunk); werr != nil {
				break
			}
			// Rule 11: the agent sees each chunk before the next is read.
			if flusher != nil {
				flusher.Flush()
			}
		}
		if rerr != nil {
			break
		}
	}

	c.Response = body.String()
	c.TotalMs = msSince(start)
	record(c)
}

func safeHeaders(h http.Header) map[string]string {
	kept := map[string]string{}
	for k, vs := range h {
		lower := strings.ToLower(k)
		if secretHeader[lower] || hopByHop[lower] || lower == "content-length" || lower == "host" {
			continue
		}
		kept[lower] = vs[0]
	}
	return kept
}

// forward sends the call upstream, retrying only transport failures — by then
// nothing has reached the agent, so a retry cannot duplicate a served response.
func forward(r *http.Request, path, wire string, body []byte) (*http.Response, int, error) {
	var lastErr error
	for attempt := 0; attempt <= maxRetries; attempt++ {
		if attempt > 0 {
			time.Sleep(time.Duration(attempt) * 250 * time.Millisecond)
		}
		// The query survives the hop: Gemini asks for a stream with ?alt=sse.
		target := base + path
		if r.URL.RawQuery != "" {
			target += "?" + r.URL.RawQuery
		}
		req, err := http.NewRequestWithContext(r.Context(), r.Method, target, bytes.NewReader(body))
		if err != nil {
			return nil, attempt, err
		}
		for k, vs := range r.Header {
			lower := strings.ToLower(k)
			if secretHeader[lower] || hopByHop[lower] || lower == "content-length" || lower == "host" {
				continue // agent credentials are placeholders; the pin changed the length
			}
			for _, v := range vs {
				req.Header.Add(k, v)
			}
		}
		req.ContentLength = int64(len(body))
		// Rule 12: the credential goes in the header the target wire expects.
		switch wire {
		case "anthropic":
			req.Header.Set("x-api-key", apiKey)
		case "gemini":
			req.Header.Set("x-goog-api-key", apiKey)
		default:
			req.Header.Set("authorization", "Bearer "+apiKey)
		}

		resp, err := client.Do(req)
		if err == nil {
			return resp, attempt, nil
		}
		lastErr = err
		if r.Context().Err() != nil {
			break // the agent gave up; don't keep retrying on its behalf
		}
	}
	return nil, maxRetries, lastErr
}

// probeHealth is the readiness probe's exit code: 0 when the edge answers on
// its own port, 1 otherwise.
func probeHealth(addr string) int {
	resp, err := http.Get("http://127.0.0.1:" + strings.TrimPrefix(addr, ":") + "/health")
	if err != nil {
		return 1
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return 1
	}
	return 0
}

// configError reports why the edge must not start. Translation is a gateway's
// job, and an operator who asked for it here has misconfigured the stack: fail
// at boot rather than at every call.
func configError() error {
	if os.Getenv("EVAL_MODEL_API") != "" {
		return errors.New("EVAL_MODEL_API is set: the edge does not translate protocols — route through a gateway, which does")
	}
	if base == "" {
		return errors.New("OPENAI_API_BASE is required")
	}
	return nil
}

func main() {
	// Rule 16: readiness is reported by the binary itself, so no shell is needed
	// and the image can be scratch.
	if len(os.Args) > 1 && os.Args[1] == "health" {
		os.Exit(probeHealth(listen))
	}

	http.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("ok"))
	})

	if err := configError(); err != nil {
		log.Fatal(err)
	}
	http.HandleFunc("/", handle)
	log.Printf("edge recording to %s (totals in %s), upstream %s", out, usagePath(), base)
	log.Fatal(http.ListenAndServe(listen, nil))
}
