package lang

import (
	"bytes"
	"testing"

	"github.com/8b-is/enthea/pure"
)

// runProg assembles and runs a program, returning its registers.
func runProg(t *testing.T, src string) (Registers, *VM) {
	t.Helper()
	prog, err := Assemble(src)
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}
	vm, err := NewVM(prog, 4096)
	if err != nil {
		t.Fatalf("newvm: %v", err)
	}
	t.Cleanup(func() { vm.Arena().Close() })
	if err := vm.Run(100000); err != nil {
		t.Fatalf("run: %v", err)
	}
	return vm.Regs(), vm
}

var letterNames = []string{"zero", "nor", "anb", "nota", "nab", "notb", "xor", "nand",
	"and", "xnor", "b", "imp", "a", "bimp", "or", "one"}

// wantCell computes the tritwise result of a letter over the two input cells
// (0 or 1): the Boolean function applies at every trit position, so with the
// Boolean living in trit0 the result is [f(a,b), f(0,0), f(0,0)].
func wantCell(f pure.F, a, b int) Cell {
	return encodeCell([3]int8{
		int8(f.Eval(a, b)),
		int8(f.Eval(0, 0)),
		int8(f.Eval(0, 0)),
	})
}

// TestLettersExhaustive — every letter × every input pair, checked against
// the shrine's canonical truth tables.
func TestLettersExhaustive(t *testing.T) {
	for f := 0; f < 16; f++ {
		for a := 0; a <= 1; a++ {
			for b := 0; b <= 1; b++ {
				src := "ldi r0 0\nldi r1 0\n"
				if a == 1 {
					src = "ldi r0 1\nldi r1 0\n"
				}
				if b == 1 {
					src = "ldi r0 0\nldi r1 1\n"
				}
				if a == 1 && b == 1 {
					src = "ldi r0 1\nldi r1 1\n"
				}
				name := letterNames[f]
				src += name + " r2 r0 r1\nhalt"
				regs, _ := runProg(t, src)
				want := wantCell(pure.F(f), a, b)
				if regs[2] != want {
					t.Fatalf("%s(%d,%d) = %d, want %d", name, a, b, regs[2], want)
				}
			}
		}
	}
	t.Log("all 16 letters × 4 inputs agree with the shrine's truth tables")
}

// TestNandInVM — a single canonical gate through the machine.
func TestNandInVM(t *testing.T) {
	regs, _ := runProg(t, "ldi r0 0\nldi r1 1\nnand r2 r0 r1\nhalt")
	if want := wantCell(pure.F(7), 0, 1); regs[2] != want {
		t.Fatalf("nand(0,1) = %d, want %d", regs[2], want)
	}
	t.Log("NAND through the VM agrees with the shrine")
}

// TestUnknownPropagation — the deterministic letters carry -1 through: the
// Bayesian interior is not dropped.
func TestUnknownPropagation(t *testing.T) {
	// a single unknown trit stays unknown in that position
	regs, _ := runProg(t, "ldi r0 -1\nldi r1 1\nand r2 r0 r1\nhalt")
	if regs[2] != -1 {
		t.Fatalf("and(unknown, 1) = %d, want -1 (unknown propagates)", regs[2])
	}
	// a fully unknown cell (-13 = three unknown trits) propagates everywhere
	regs, _ = runProg(t, "ldi r0 -13\nldi r1 1\nand r2 r0 r1\nhalt")
	if regs[2] != -13 {
		t.Fatalf("and(unknown³, 1) = %d, want -13", regs[2])
	}
	t.Log("-1 propagates through the letters: unknown stays unknown")
}

// TestTernaryArithmetic — balanced-ternary add and multiply.
func TestTernaryArithmetic(t *testing.T) {
	regs, _ := runProg(t, "ldi r0 2\nldi r1 -3\ntadd r2 r0 r1\nldi r0 3\nldi r1 -2\ntmul r3 r0 r1\nhalt")
	if regs[2] != -1 {
		t.Fatalf("2 + (-3) = %d, want -1", regs[2])
	}
	if regs[3] != -6 {
		t.Fatalf("3 × (-2) = %d, want -6", regs[3])
	}
	t.Log("balanced ternary add/mul correct")
}

// TestRecursiveFactorial — CALL/RET + PUSH/POP recursion on the arena's own
// call stack: fact(3) = 6, proven by the machine itself. The call stack is
// the only stack — locals live on it, in the same arena as the program.
func TestRecursiveFactorial(t *testing.T) {
	src := `
main:
  ldi r0 3
  call fact
  halt
fact:
  jz r0 base
  push r0
  ldi r1 -1
  tadd r0 r0 r1
  call fact
  pop r1
  tmul r0 r0 r1
  ret
base:
  ldi r0 1
  ret
`
	regs, vm := runProg(t, src)
	if regs[0] != 6 {
		t.Fatalf("fact(3) = %d, want 6", regs[0])
	}
	if vm.StackPeak() < 1 {
		t.Fatalf("the call stack should have been used (peak %d)", vm.StackPeak())
	}
	t.Log("fact(3)=6; the arena's own call stack holds returns AND locals — one region")
}

// TestArenaIsTheWorld — the bytecode sits at the front of the arena and the
// call stack at the far end: one region, everything in it.
func TestArenaIsTheWorld(t *testing.T) {
	prog, err := Assemble("call sub\nhalt\nsub:\nret")
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}
	vm, err := NewVM(prog, 2048)
	if err != nil {
		t.Fatalf("newvm: %v", err)
	}
	t.Cleanup(func() { vm.Arena().Close() })
	view := vm.Arena().View()
	if !bytes.Equal(view[:len(prog)], prog) {
		t.Fatal("the program is not at the front of the arena")
	}
	if err := vm.Run(1000); err != nil {
		t.Fatalf("run: %v", err)
	}
	if !vm.halt {
		t.Fatal("the program did not halt cleanly")
	}
	if vm.StackPeak() < 1 {
		t.Fatalf("the call stack should have been used (peak %d)", vm.StackPeak())
	}
	if vm.Arena().IsMmap() {
		t.Log("arena is a real kernel mapping — stage 3 already starting")
	}
	t.Log("one arena: program at the front, stack at the far end, everything owned")
}
