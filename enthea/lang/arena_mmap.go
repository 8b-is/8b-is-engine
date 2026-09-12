//go:build unix

package lang

import (
	"fmt"
	"syscall"
)

// NewArena maps `size` bytes from the kernel: one real mapping, the stage-3
// path — no libc, no malloc, just the page tables.
func NewArena(size int) (*Arena, error) {
	if size <= 0 {
		return nil, fmt.Errorf("lang: arena size must be positive")
	}
	m, err := syscall.Mmap(-1, 0, size, syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_PRIVATE|syscall.MAP_ANON)
	if err != nil {
		return nil, fmt.Errorf("lang: mmap %d bytes: %w", size, err)
	}
	return &Arena{data: m, mmap: true}, nil
}

// Close releases the mapping.
func (a *Arena) Close() error {
	if a.mmap {
		err := syscall.Munmap(a.data)
		a.data = nil
		return err
	}
	a.data = nil
	return nil
}

// IsMmap reports whether the arena is a real kernel mapping (the stage-3
// path) or the Go-slice fallback.
func (a *Arena) IsMmap() bool { return a.mmap }
