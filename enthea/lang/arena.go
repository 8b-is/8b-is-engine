// Package lang — the enthea language (ANDROMEDA stage 1).
//
// The whole world is one arena: program, heap, and call stack share a single
// region that the VM owns. On unix the arena is a kernel mmap; elsewhere it
// falls back to an owned byte slice. The arena's DATA plane is typed: cells
// are ternary {-1,0,+1} (the MLX-QUANT kernel's own alphabet), and the
// kernel ops read and write cells directly — no byte↔cell casting. What the
// language may express is the capability layer's call (vakedc), not the
// memory's: the arena stores what it is told, the capability graph decides
// what can be told.
package lang

import "fmt"

// Arena is a single owned region with a bump allocator. The backing store is
// bytes; the data plane is exposed as cells.
type Arena struct {
	data []byte
	mmap bool
	off  int
	peak int
}

// Alloc bumps `n` bytes and returns their offset.
func (a *Arena) Alloc(n int) (int, error) {
	if n < 0 {
		return 0, fmt.Errorf("lang: negative allocation %d", n)
	}
	if a.off+n > len(a.data) {
		return 0, fmt.Errorf("lang: arena exhausted (%d used, %d wanted of %d)", a.off, n, len(a.data))
	}
	off := a.off
	a.off += n
	if a.off > a.peak {
		a.peak = a.off
	}
	return off, nil
}

// View is the full arena as one byte slice.
func (a *Arena) View() []byte { return a.data }

// CellAt reads the ternary cell at `addr` — the quantized data plane.
func (a *Arena) CellAt(addr int) Cell {
	if addr < 0 || addr >= len(a.data) {
		return 0
	}
	return Cell(int8(a.data[addr]))
}

// SetCell writes the ternary cell at `addr`, quantized to the alphabet.
func (a *Arena) SetCell(addr int, c Cell) {
	if addr < 0 || addr >= len(a.data) {
		return
	}
	a.data[addr] = byte(quant(c))
}

// quant snaps a value to the ternary alphabet {-1,0,+1} — the kernel's
// own precision. Values on the boundary round toward zero.
func quant(c Cell) Cell {
	switch {
	case c > 0:
		return 1
	case c < 0:
		return -1
	default:
		return 0
	}
}

// Stats reports (total, used, peak) in bytes.
func (a *Arena) Stats() (total, used, peak int) {
	return len(a.data), a.off, a.peak
}
