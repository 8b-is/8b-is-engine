package lang

import "testing"

// TestQuantizedArena — the data plane is ternary: writing a value through
// SetCell snaps it to the kernel's alphabet {-1,0,+1}.
func TestQuantizedArena(t *testing.T) {
	a, err := NewArena(256)
	if err != nil {
		t.Fatalf("newarena: %v", err)
	}
	defer a.Close()
	a.SetCell(5, 13) // saturating + value → +1
	if a.CellAt(5) != 1 {
		t.Fatalf("CellAt(5) = %d, want 1", a.CellAt(5))
	}
	a.SetCell(6, -7) // negative → -1
	if a.CellAt(6) != -1 {
		t.Fatalf("CellAt(6) = %d, want -1", a.CellAt(6))
	}
	a.SetCell(7, 0)
	if a.CellAt(7) != 0 {
		t.Fatalf("CellAt(7) = %d, want 0", a.CellAt(7))
	}
	t.Log("the arena's data plane is the ternary alphabet — the kernel's own memory")
}

// TestDynamicRegisters — the register matrix grows on demand: r20 works
// without being declared up front.
func TestDynamicRegisters(t *testing.T) {
	regs, _ := runProg(t, "ldi r20 3\nldi r21 2\ntadd r22 r20 r21\nhalt")
	if regs[22] != 5 {
		t.Fatalf("r22 = %d, want 5 (r20/r21 beyond the initial 16)", regs[22])
	}
	t.Log("the register matrix is dynamically resizable — r20+ just works")
}

// TestQmatKernel — the MLX-QUANT matrix kernel: a 2x3 register matrix
// against ternary weights, per-row qdot + ultra into the first rows.
func TestQmatKernel(t *testing.T) {
	src := `
main:
  ldi r4 1
  ldi r5 -1
  ldi r6 0
  ldi r7 1
  ldi r8 1
  ldi r9 -1
  qmat r4 2 3 weights
  halt
weights:
  .byte 1 -1 1
`
	regs, _ := runProg(t, src)
	// row0: 1·1 + (-1)·(-1) + 0·1 = 2 → ultra → +1
	// row1: 1·1 + 1·(-1) + (-1)·1 = -1 → ultra → -1
	if regs[4] != 1 {
		t.Fatalf("row0 activation = %d, want 1", regs[4])
	}
	if regs[5] != -1 {
		t.Fatalf("row1 activation = %d, want -1", regs[5])
	}
	t.Log("qmat: the register matrix × ternary weights → quantized activations")
}
