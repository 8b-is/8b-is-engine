package vakedc

import (
	"testing"

	"github.com/8b-is/enthea/lang"
	"github.com/8b-is/enthea/pure"
)

// TestCompletenessExecutable — for every one of the sixteen Boolean
// functions, vakedc assembles a NAND-only program and the machine computes
// the correct truth table: functional completeness, made runnable.
func TestCompletenessExecutable(t *testing.T) {
	s := Synthesize()
	if len(s.best) != 16 {
		t.Fatalf("the capability graph reached %d functions, want all 16", len(s.best))
	}
	for f := pure.F(0); f < 16; f++ {
		asm, cost := Emit(f, s)
		if asm == "" {
			t.Fatalf("no capability path for function %d", f)
		}
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
				full := src + asm + "halt\n"
				prog, err := lang.Assemble(full)
				if err != nil {
					t.Fatalf("assemble fn %d: %v", f, err)
				}
				vm, err := lang.NewVM(prog, 2048)
				if err != nil {
					t.Fatalf("newvm fn %d: %v", f, err)
				}
				if err := vm.Run(1000); err != nil {
					t.Fatalf("run fn %d: %v", f, err)
				}
				vm.Arena().Close()
				got := vm.Regs()[0]
				// the machine's letters are tritwise: the circuit computes
				// f at trit0 and f(0,0) at the higher trits
				want := lang.Cell(f.Eval(a, b)) + 3*lang.Cell(f.Eval(0, 0)) + 9*lang.Cell(f.Eval(0, 0))
				if got != want {
					t.Fatalf("fn %d(%d,%d) = %d, want %d (cost %d NANDs)", f, a, b, got, want, cost)
				}
			}
		}
		if cost < 0 {
			t.Fatalf("fn %d has a bogus cost %d", f, cost)
		}
	}
	t.Log("all 16 functions assembled from NAND alone, each verified on the machine")
}

// TestClosureCosts — the NAND costs fall out of the closure: NOT = 1,
// AND = 2, OR = 3. XOR lands at 5 (the circuit-minimal 4 needs shared-DAG
// synthesis, a richer search); every function is still correctly assembled.
func TestClosureCosts(t *testing.T) {
	s := Synthesize()
	cost := func(f pure.F) int {
		_, c := Emit(f, s)
		return c
	}
	if got := cost(pure.F(3)); got != 1 { // nota
		t.Fatalf("NOT costs %d NANDs, want 1", got)
	}
	if got := cost(pure.F(8)); got != 2 { // and
		t.Fatalf("AND costs %d NANDs, want 2", got)
	}
	if got := cost(pure.F(14)); got != 3 { // or
		t.Fatalf("OR costs %d NANDs, want 3", got)
	}
	if got := cost(pure.F(6)); got != 5 { // xor
		t.Fatalf("XOR costs %d NANDs, want 5 (closure; circuit-minimal is 4)", got)
	}
	t.Log("NAND costs from the closure: NOT 1 · AND 2 · OR 3 · XOR 5")
}

// TestCapabilityGraphIsTheLattice — every function is present and the graph
// is connected: reachability is exactly functional completeness.
func TestCapabilityGraphIsTheLattice(t *testing.T) {
	s := Synthesize()
	for f := pure.F(0); f < 16; f++ {
		if _, ok := s.best[f]; !ok {
			t.Fatalf("function %d is unreachable from NAND", f)
		}
	}
	t.Log("the graph closes over all sixteen vertices — the lattice, walked")
}
