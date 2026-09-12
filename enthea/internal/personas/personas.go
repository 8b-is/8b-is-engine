// Package personas embeds the constellation voices and exposes them as
// installable agent/skill files. Pure stdlib: go:embed + io/fs.
package personas

import (
	"embed"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

//go:embed *.md
var files embed.FS

// Persona is one constellation voice.
type Persona struct {
	// Name is the voice id (directory/folder safe), e.g. "al-biruni".
	Name string
	// Title is the human title, e.g. "Al-Biruni — the polyhistor".
	Title string
	// Body is the full persona text.
	Body string
}

// List returns every embedded persona, sorted by name. It walks the embed
// FS once (io/fs.ReadDir + ReadFile) — the idiomatic embed-to-slice shape.
func List() ([]Persona, error) {
	entries, err := fs.ReadDir(files, ".")
	if err != nil {
		return nil, fmt.Errorf("list personas: %w", err)
	}
	ps := make([]Persona, 0, len(entries))
	for _, e := range entries {
		if e.IsDir() || filepath.Ext(e.Name()) != ".md" {
			continue
		}
		body, err := files.ReadFile(e.Name())
		if err != nil {
			return nil, fmt.Errorf("read persona %s: %w", e.Name(), err)
		}
		ps = append(ps, Persona{
			Name:  strings.TrimSuffix(e.Name(), ".md"),
			Title: firstLine(string(body)),
			Body:  string(body),
		})
	}
	sort.Slice(ps, func(i, j int) bool { return ps[i].Name < ps[j].Name })
	return ps, nil
}

// Get returns the persona with the given name, or an error.
func Get(name string) (Persona, error) {
	ps, err := List()
	if err != nil {
		return Persona{}, err
	}
	for _, p := range ps {
		if p.Name == name {
			return p, nil
		}
	}
	return Persona{}, fmt.Errorf("persona %q not found (have: %s)", name, strings.Join(names(ps), ", "))
}

// Install writes every persona into dir as an opencode-compatible agent file
// (`<dir>/<name>.md` with an `# Title` body). Returns the written paths.
func Install(dir string) ([]string, error) {
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, fmt.Errorf("mkdir %s: %w", dir, err)
	}
	ps, err := List()
	if err != nil {
		return nil, fmt.Errorf("list personas: %w", err)
	}
	var written []string
	for _, p := range ps {
		path := filepath.Join(dir, p.Name+".md")
		if err := os.WriteFile(path, []byte(p.Body), 0o644); err != nil {
			return nil, fmt.Errorf("write %s: %w", path, err)
		}
		written = append(written, path)
	}
	return written, nil
}

// ErrNoPersonas guards callers that need at least one voice.
var ErrNoPersonas = errors.New("no personas embedded")

// --- helpers ---

func names(ps []Persona) []string {
	out := make([]string, len(ps))
	for i, p := range ps {
		out[i] = p.Name
	}
	return out
}

// firstLine strips the leading `# ` title from a persona body.
func firstLine(s string) string {
	if i := strings.IndexByte(s, '\n'); i >= 0 {
		s = s[:i]
	}
	return strings.TrimPrefix(strings.TrimSpace(s), "# ")
}
