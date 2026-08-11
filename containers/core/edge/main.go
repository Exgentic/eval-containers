// The edge: the one component every model call crosses (.agents/edge/RULES.md).
//
// It pins the model to EVAL_MODEL (rule 2), forwards the call on the wire it
// arrived on (rule 4), streams the response back unbuffered (rule 11), and
// records the exchange as the agent sent it (rules 6-8). The upstream
// credential it injects never enters a record (rule 9).
//
// Stdlib only, so the binary is static and runs with no runtime dependency
// (rule 15): the same file is a scratch image, a sidecar, and a process inside
// the standalone bundle.
package main

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"
)

// A record of one call. Headers are captured minus every credential-bearing
// one, so rule 9 holds by construction rather than by redaction.
type call struct {
	Path      string            `json:"path"`
	Wire      string            `json:"wire"`
	StartUnix float64           `json:"start_unix"`
	Headers   map[string]string `json:"headers"`
	Request   string            `json:"request"`  // agent-verbatim, before the pin
	Upstream  string            `json:"upstream"` // what was actually sent
	Status    int               `json:"status"`
	Response  string            `json:"response"`
	TTFTms    float64           `json:"ttft_ms"`
	TotalMs   float64           `json:"total_ms"`
	Chunks    [][2]float64      `json:"chunks"` // [ms since request start, bytes]
	Truncated bool              `json:"truncated,omitempty"`
	Retries   int               `json:"retries,omitempty"`
}

var (
	mu     sync.Mutex
	out    = envOr("OUT", "/output/calls.jsonl")
	listen = envOr("LISTEN", ":4000")
	model  = os.Getenv("EVAL_MODEL")
	base   = strings.TrimSuffix(os.Getenv("OPENAI_API_BASE"), "/")
	apiKey = os.Getenv("OPENAI_API_KEY")

	// No client timeout: an agent turn legitimately runs for minutes.
	client = &http.Client{}

	wirePrefix = map[string]string{"anthropic": "/anthropic", "gemini": "/genai", "openai": "/openai"}
	// Credentials never reach a record (rule 9); the agent's are placeholders anyway.
	secretHeader = map[string]bool{"authorization": true, "x-api-key": true, "x-goog-api-key": true}
	// Gemini names the model in the URL rather than the body.
	geminiModel = regexp.MustCompile(`/models/[^:/]+(:|$)`)

	// Cap what one record may hold so a multimodal payload can't exhaust memory.
	maxRecord = envInt("EDGE_MAX_RECORD_BYTES", 8<<20)
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

func record(c call) {
	mu.Lock()
	defer mu.Unlock()
	f, err := os.OpenFile(out, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		log.Println("record:", err)
		return
	}
	defer f.Close()
	if err := json.NewEncoder(f).Encode(c); err != nil {
		log.Println("record:", err)
	}
}

// pin replaces the model the agent named with EVAL_MODEL. The field is
// top-level JSON on every body-carrying wire, so no per-wire knowledge is
// needed — and the handle is never parsed (rule 3).
func pin(body []byte) []byte {
	if model == "" || len(body) == 0 {
		return body
	}
	var m map[string]any
	if err := json.Unmarshal(body, &m); err != nil {
		return body // not JSON we understand: forward untouched
	}
	m["model"] = model
	pinned, err := json.Marshal(m)
	if err != nil {
		return body
	}
	return pinned
}

func handle(w http.ResponseWriter, r *http.Request) {
	start := time.Now()

	// Rule 5: translation is a gateway's job, and silence about it is forbidden.
	if os.Getenv("EVAL_MODEL_API") != "" {
		http.Error(w, `{"error":{"type":"unsupported_wire","message":"the edge does not translate protocols: unset EVAL_MODEL_API, or route through a translating gateway"}}`,
			http.StatusNotImplemented)
		return
	}

	wire, path := "openai", r.URL.Path
	for name, prefix := range wirePrefix {
		if strings.HasPrefix(path, prefix+"/") {
			wire, path = name, strings.TrimPrefix(path, prefix)
			break
		}
	}

	agentBody, _ := io.ReadAll(r.Body)
	upstreamBody := agentBody
	if wire == "gemini" {
		// The handle goes in verbatim, slashes included (rule 3).
		if model != "" {
			path = geminiModel.ReplaceAllString(path, "/models/"+model+"$1")
		}
	} else {
		upstreamBody = pin(agentBody)
	}

	c := call{
		Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
		Headers: map[string]string{},
	}
	c.Request = clip(agentBody, &c.Truncated)
	c.Upstream = clip(upstreamBody, &c.Truncated)

	resp, retries, err := forward(r, path, wire, upstreamBody, c.Headers)
	c.Retries = retries
	if err != nil {
		http.Error(w, `{"error":{"type":"upstream_unreachable","message":"`+err.Error()+`"}}`, http.StatusBadGateway)
		c.Status = http.StatusBadGateway
		c.TotalMs = msSince(start)
		record(c)
		return
	}
	defer resp.Body.Close()

	for k, vs := range resp.Header {
		for _, v := range vs {
			w.Header().Add(k, v)
		}
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
	if len(c.Chunks) > 0 {
		c.TTFTms = c.Chunks[0][0]
	}
	record(c)
}

// forward sends the call upstream, retrying only transport failures — by then
// nothing has reached the agent, so a retry cannot duplicate a served response.
func forward(r *http.Request, path, wire string, body []byte, captured map[string]string) (*http.Response, int, error) {
	var lastErr error
	for attempt := 0; attempt <= maxRetries; attempt++ {
		if attempt > 0 {
			time.Sleep(time.Duration(attempt) * 250 * time.Millisecond)
		}
		req, err := http.NewRequestWithContext(r.Context(), r.Method, base+path, bytes.NewReader(body))
		if err != nil {
			return nil, attempt, err
		}
		for k, vs := range r.Header {
			lower := strings.ToLower(k)
			if secretHeader[lower] || lower == "content-length" || lower == "host" {
				continue // agent credentials are placeholders; the pin changed the length
			}
			for _, v := range vs {
				req.Header.Add(k, v)
				captured[lower] = v
			}
		}
		req.Header.Set("content-length", strconv.Itoa(len(body)))
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

func clip(b []byte, truncated *bool) string {
	if len(b) > maxRecord {
		*truncated = true
		return string(b[:maxRecord])
	}
	return string(b)
}

func main() {
	// Rule 16: readiness is reported by the binary itself, so no shell is needed
	// and the image can be scratch.
	if len(os.Args) > 1 && os.Args[1] == "health" {
		resp, err := http.Get("http://127.0.0.1:" + strings.TrimPrefix(listen, ":") + "/health")
		if err != nil || resp.StatusCode != http.StatusOK {
			os.Exit(1)
		}
		return
	}
	if base == "" {
		log.Fatal("OPENAI_API_BASE is required")
	}
	http.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("ok"))
	})
	http.HandleFunc("/", handle)
	log.Printf("edge listening on %s -> %s", listen, base)
	log.Fatal(http.ListenAndServe(listen, nil))
}
