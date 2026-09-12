//go:build !unix

package lang

import "fmt"

// NewArena falls back to an owned byte slice on platforms without mmap in
// the standard library.
func NewArena(size int) (*Arena, error) {
	if size <= 0 {
		return nil, fmt.Errorf("lang: arena size must be positive")
	}
	return &Arena{data: make([]byte, size)}, nil
}

// Close is a no-op for the fallback slice.
func (a *Arena) Close() error {
	a.data = nil
	return nil
}

// IsMmap reports whether the arena is a real kernel mapping.
func (a *Arena) IsMmap() bool { return false }
