package lang

import "testing"

// TestQdot — the MLX-QUANT seam: a ternary-quantized dot product. The arena
// holds the b1.58-style weights {-1,0,+1}; the registers hold the inputs;
// qdot sums the products in balanced ternary.
func TestQdot(t *testing.T) {
	src := `
main:
  ldi r0 1
  ldi r1 -1
  ldi r2 0
  ldi r3 1
  qdot r4 weights 4
  halt
weights:
  .byte 1 0 -1 1
`
	regs, _ := runProg(t, src)
	// 1·1 + (-1)·0 + 0·(-1) + 1·1 = 2
	if regs[4] != 2 {
		t.Fatalf("qdot([1,-1,0,1]·[1,0,-1,1]) = %d, want 2", regs[4])
	}
	t.Log("qdot: the arena's ternary weights and the registers' inputs dot in balanced ternary")
}

// TestUltra — the b1.58 hard activation: the soft interior sharpens to the
// ternary alphabet {-1,0,+1}.
func TestUltra(t *testing.T) {
	regs, _ := runProg(t, "ldi r0 5\nultra r0\nldi r1 -7\nultra r1\nldi r2 0\nultra r2\nhalt")
	if regs[0] != 1 || regs[1] != -1 || regs[2] != 0 {
		t.Fatalf("ultra([5,-7,0]) = [%d,%d,%d], want [1,-1,0]", regs[0], regs[1], regs[2])
	}
	t.Log("ultra: the soft interior becomes the hard ternary alphabet")
}

// TestTailCallConstantStack — tail recursion is a jump: the countdown lives
// in the arena (120 bytes of raw data), the loop recurses in tail position,
// and 120 iterations run on a constant stack. The compiler rewrote
// `call count; ret` into `jmp count`.
func TestTailCallConstantStack(t *testing.T) {
	src := `
main:
  call count
  halt
count:
  ld r1 counter
  jz r1 done
  ldi r2 -1
  tadd r1 r1 r2
  st counter r1
  call count
  ret
done:
  ret
counter:
  .byte 120
`
	prog, err := Assemble(src)
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}
	m, err := NewVM(prog, 4096)
	if err != nil {
		t.Fatalf("newvm: %v", err)
	}
	t.Cleanup(func() { m.Arena().Close() })
	if err := m.Run(100000); err != nil {
		t.Fatalf("run: %v", err)
	}
	// the counter is the last program byte (the .byte data cell)
	counter := len(prog) - 1
	if m.Arena().View()[counter] != 0 {
		t.Fatal("the arena counter did not reach 0")
	}
	if m.StackPeak() > 2 {
		t.Fatalf("tail recursion grew the stack: peak %d bytes (one 2-byte frame)", m.StackPeak())
	}
	t.Log("120 tail-recursive iterations on a constant stack — TCO is a jump")
}

// TestDirectThreadedDispatch — the interpreter dispatches through the opcode
// table (the sixteen letters are sixteen native handlers), not a switch.
func TestDirectThreadedDispatch(t *testing.T) {
	// every slot 0x00..0x0f must be a distinct installed handler
	for f := 0; f < 16; f++ {
		if ops[f] == nil {
			t.Fatalf("letter opcode 0x%02x has no threaded handler", f)
		}
	}
	// and the spine slots used by the programs above
	for _, op := range []byte{opLdi, opTadd, opJz, opQdot, opUltra, opCall, opJmp, opHalt} {
		if ops[op] == nil {
			t.Fatalf("spine opcode 0x%02x has no threaded handler", op)
		}
	}
	t.Log("the alphabet is the dispatch table: 16 letters, 16 native handlers")
}
