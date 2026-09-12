// Package mcp: a minimal Model Context Protocol server over stdio, built on
// nothing but the standard library. It speaks line-delimited JSON-RPC 2.0 and
// serves a set of Tools over an injectable io.ReadWriteCloser, so the same
// server can run over stdin/stdout or an in-memory pipe in tests.
package mcp

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"sync"
)

// Tool is one MCP tool: a name, a human description, an input JSON schema
// (free-form map), and a Run that returns a JSON-serializable result.
type Tool interface {
	Name() string
	Description() string
	InputSchema() map[string]any
	Run(ctx context.Context, args map[string]any) (any, error)
}

// Server holds the registered tools and serves the protocol on a stream.
type Server struct {
	tools map[string]Tool // keyed by name — collision = config bug, fail early
	mu    sync.Mutex      // guards tools while the serve loop reads them
}

// New builds a Server and registers the given tools. Duplicate names error.
func New(tools ...Tool) (*Server, error) {
	s := &Server{tools: make(map[string]Tool, len(tools))}
	for _, t := range tools {
		if _, dup := s.tools[t.Name()]; dup {
			return nil, fmt.Errorf("mcp: duplicate tool %q", t.Name())
		}
		s.tools[t.Name()] = t
	}
	return s, nil
}

// Tools returns the registered tools as a slice (for tools/list).
func (s *Server) Tools() []Tool {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([]Tool, 0, len(s.tools))
	for _, t := range s.tools {
		out = append(out, t)
	}
	return out
}

// --- JSON-RPC wire types ---

type request struct {
	JSONRPC string         `json:"jsonrpc"`
	ID      any            `json:"id"`
	Method  string         `json:"method"`
	Params  map[string]any `json:"params,omitempty"`
}

type response struct {
	JSONRPC string  `json:"jsonrpc"`
	ID      any     `json:"id"`
	Result  any     `json:"result,omitempty"`
	Error   *rpcErr `json:"error,omitempty"`
}

type rpcErr struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

const (
	codeParse  = -32700
	codeMethod = -32601
	codeCall   = -32602
)

// Serve reads line-delimited JSON-RPC requests from rw until EOF and writes
// responses back. It returns the first I/O error encountered after draining.
func (s *Server) Serve(ctx context.Context, rw io.ReadWriteCloser) error {
	defer rw.Close()
	sc := bufio.NewScanner(rw)
	enc := json.NewEncoder(rw)
	sc.Buffer(make([]byte, 0, 64*1024), 4*1024*1024)
	for sc.Scan() {
		line := sc.Bytes()
		if len(line) == 0 {
			continue
		}
		var req request
		if err := json.Unmarshal(line, &req); err != nil {
			// Notification-shaped malformed input: still respond with parse error.
			_ = enc.Encode(response{JSONRPC: "2.0", Error: &rpcErr{Code: codeParse, Message: "parse error"}})
			continue
		}
		_ = enc.Encode(s.handle(ctx, req))
	}
	if err := sc.Err(); err != nil && !errors.Is(err, io.EOF) {
		return fmt.Errorf("mcp: read: %w", err)
	}
	return nil
}

// handle routes one request to the protocol or a tool and builds the reply.
func (s *Server) handle(ctx context.Context, req request) response {
	resp := response{JSONRPC: "2.0", ID: req.ID}
	switch req.Method {
	case "initialize":
		resp.Result = map[string]any{
			"protocolVersion": "2024-11-05",
			"capabilities":    map[string]any{"tools": map[string]any{}},
			"serverInfo":      map[string]any{"name": "enthea", "version": "0.1.0"},
		}
	case "notifications/initialized":
		// no reply for notifications
		return response{}
	case "tools/list":
		resp.Result = map[string]any{"tools": s.toolList()}
	case "tools/call":
		name, _ := req.Params["name"].(string)
		args, _ := req.Params["arguments"].(map[string]any)
		tool, ok := s.tools[name]
		if !ok {
			resp.Error = &rpcErr{Code: codeMethod, Message: "unknown tool: " + name}
			return resp
		}
		result, err := tool.Run(ctx, args)
		if err != nil {
			resp.Error = &rpcErr{Code: codeCall, Message: err.Error()}
			return resp
		}
		resp.Result = map[string]any{
			"content": []map[string]any{{
				"type": "text",
				"text": mustJSON(result),
			}},
		}
	default:
		resp.Error = &rpcErr{Code: codeMethod, Message: "method not found: " + req.Method}
	}
	return resp
}

func (s *Server) toolList() []map[string]any {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([]map[string]any, 0, len(s.tools))
	for _, t := range s.tools {
		out = append(out, map[string]any{
			"name":        t.Name(),
			"description": t.Description(),
			"inputSchema": t.InputSchema(),
		})
	}
	return out
}

func mustJSON(v any) string {
	b, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return fmt.Sprintf("{\"error\": %q}", err.Error())
	}
	return string(b)
}
