// Package mcp: the shipped enthea tools — personas, kompress, engine status.
package mcp

import (
	"context"
	"fmt"
	"os/exec"
	"regexp"
	"strings"

	"github.com/8b-is/enthea/internal/personas"
)

// --- personas_list ---

// PersonasTool lists the constellation voices from the embedded set.
type PersonasTool struct{}

func (PersonasTool) Name() string { return "personas_list" }
func (PersonasTool) Description() string {
	return "List the constellation voices (Al-Biruni and the essences)."
}
func (PersonasTool) InputSchema() map[string]any {
	return map[string]any{"type": "object", "properties": map[string]any{}}
}

// Run lists the personas as [{name, title}].
func (PersonasTool) Run(_ context.Context, _ map[string]any) (any, error) {
	ps, err := personas.List()
	if err != nil {
		return nil, fmt.Errorf("list personas: %w", err)
	}
	out := make([]map[string]string, 0, len(ps))
	for _, p := range ps {
		out = append(out, map[string]string{"name": p.Name, "title": p.Title})
	}
	return map[string]any{"persons": out}, nil
}

// --- kompress_compress ---

// Shell is the injected command runner; os/exec by default, swapped in tests.
type Shell interface {
	Run(name string, args ...string) (string, error)
}

// OSShell runs a command and returns its trimmed stdout.
type OSShell struct{}

func (OSShell) Run(name string, args ...string) (string, error) {
	out, err := exec.Command(name, args...).CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("%s: %w: %s", name, err, strings.TrimSpace(string(out)))
	}
	return strings.TrimSpace(string(out)), nil
}

// KompressTool compresses text through the kompress-ultra CLI.
type KompressTool struct{ sh Shell }

// NewKompressTool wires the kompress tool to a command runner.
func NewKompressTool(sh Shell) *KompressTool { return &KompressTool{sh: sh} }

func (t *KompressTool) Name() string { return "kompress_compress" }
func (t *KompressTool) Description() string {
	return "Compress a text with the kompress-ultra engine and report ratio + units."
}
func (t *KompressTool) InputSchema() map[string]any {
	return map[string]any{
		"type": "object",
		"properties": map[string]any{
			"text": map[string]any{"type": "string", "description": "The text to compress."},
		},
		"required": []string{"text"},
	}
}

var ratioRe = regexp.MustCompile(`Compression ratio:\s*(\d+)%`)

func (t *KompressTool) Run(_ context.Context, args map[string]any) (any, error) {
	text, _ := args["text"].(string)
	if text == "" {
		return nil, fmt.Errorf("kompress_compress: text is required")
	}
	out, err := t.sh.Run("kompress", "compress", text)
	if err != nil {
		return nil, fmt.Errorf("kompress compress: %w", err)
	}
	ratio := 0
	if m := ratioRe.FindStringSubmatch(out); len(m) == 2 {
		fmt.Sscanf(m[1], "%d", &ratio)
	}
	var units []string
	for _, line := range strings.Split(out, "\n") {
		if rest, ok := strings.CutPrefix(line, "Unit "); ok {
			if i := strings.IndexByte(rest, ':'); i >= 0 {
				units = append(units, strings.TrimSpace(rest[i+1:]))
			}
		}
	}
	return map[string]any{"ratio_percent": ratio, "units": units}, nil
}

// --- kompress_persons ---

// PersonsTool lists the kompress-ultra brain voices.
type PersonsTool struct{ sh Shell }

// NewPersonsTool wires the persons tool to a command runner.
func NewPersonsTool(sh Shell) *PersonsTool { return &PersonsTool{sh: sh} }

func (t *PersonsTool) Name() string { return "kompress_persons" }
func (t *PersonsTool) Description() string {
	return "List the kompress-ultra brain voices (RALPH, LODRI, KRENGEL, PETER, COSMOS)."
}
func (t *PersonsTool) InputSchema() map[string]any {
	return map[string]any{"type": "object", "properties": map[string]any{}}
}

func (t *PersonsTool) Run(_ context.Context, _ map[string]any) (any, error) {
	out, err := t.sh.Run("kompress", "persons")
	if err != nil {
		return nil, fmt.Errorf("kompress persons: %w", err)
	}
	var persons []map[string]string
	for _, line := range strings.Split(out, "\n") {
		if name, role, ok := strings.Cut(line, ":"); ok {
			persons = append(persons, map[string]string{"name": strings.TrimSpace(name), "role": strings.TrimSpace(role)})
		}
	}
	return map[string]any{"persons": persons}, nil
}
