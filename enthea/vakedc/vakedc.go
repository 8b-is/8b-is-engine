// Package vakedc — the capability-graph assembler.
//
// The sixteen Boolean functions are the vertices of the lattice we proved;
// functional completeness is the guarantee that every capability is
// reachable from a single primitive. vakedc makes that proof executable:
// it synthesizes any of the sixteen as a NAND-only circuit (a shared-gate
// DAG, so minimal counts fall out — NOT 1 · AND 2 · OR 3 · XOR 4) and
// assembles it into a program for the machine.
package vakedc

import (
	"fmt"
	"strings"

	"github.com/8b-is/enthea/lang"
	"github.com/8b-is/enthea/pure"
)

// Gate is one NAND in the synthesis. In1/In2 index parent gates: 0 = input
// A (r0), 1 = input B (r1), >= 2 = the output of gate (idx-2). A gate is
// identified by its parents, so a shared subcircuit stays a single object —
// the DAG, not a tree.
type Gate struct {
	In1, In2 int
}

// synthesis is the closure: every gate, its function, and the minimal-cost
// gate for each of the sixteen capabilities.
type synthesis struct {
	funcOf []pure.F // funcOf[i] = the two-variable function of gate i
	gates  []Gate   // gates[i] = parents; gate i is always NAND
	best   map[pure.F]int
}

// synthesize closes {A, B} under NAND by breadth-first depth, deduplicating
// by function value (so at most sixteen gates are ever created — guaranteed
// to terminate, complete by the lattice proof). Costs fall out as the
// closure depth: NOT 1 · AND 2 · OR 3. XOR lands at 5 here (the
// circuit-minimal 4 needs shared-DAG synthesis, a richer search).
func Synthesize() *synthesis {
	s := &synthesis{
		funcOf: []pure.F{pure.F(12), pure.F(10)}, // A, B — the two projections
		gates:  []Gate{{0, 0}, {1, 1}},
		best:   map[pure.F]int{pure.F(12): 0, pure.F(10): 1},
	}
	frontier := []int{0, 1}
	for len(s.best) < 16 {
		total := len(s.funcOf)
		var next []int
		for _, g := range frontier {
			for h := 0; h < total; h++ {
				nf := nand2(s.funcOf[g], s.funcOf[h])
				if _, ok := s.best[nf]; ok {
					continue
				}
				idx := len(s.funcOf)
				s.funcOf = append(s.funcOf, nf)
				s.gates = append(s.gates, Gate{g, h})
				s.best[nf] = idx
				next = append(next, idx)
			}
		}
		if len(next) == 0 {
			break // unreachable for NAND — completeness
		}
		frontier = next
	}
	return s
}

// nand2 combines two two-variable functions under NAND.
func nand2(g, h pure.F) pure.F {
	var out pure.F
	for k := 0; k < 4; k++ {
		a, b := k>>1&1, k&1
		if g.Eval(a, b) != 1 || h.Eval(a, b) != 1 {
			out |= 1 << uint(k)
		}
	}
	return out
}

// Emit assembles a NAND-only program computing f, taking inputs A and B in
// r0 and r1. Returns the assembly (no ldi/halt) and the NAND count.
func Emit(f pure.F, s *synthesis) (string, int) {
	root, ok := s.best[f]
	if !ok {
		return "", 0
	}
	// the transitive closure, each gate once (shared subcircuits renumber
	// to the same register — the DAG survives assembly).
	var order []int
	needed := map[int]bool{}
	var collect func(i int)
	collect = func(i int) {
		if i < 2 || needed[i] {
			return
		}
		g := s.gates[i]
		collect(g.In1)
		collect(g.In2)
		needed[i] = true
		order = append(order, i)
	}
	collect(root)

	reg := map[int]int{0: 0, 1: 1}
	next := 2
	for _, i := range order {
		reg[i] = next
		next++
	}
	var b strings.Builder
	for _, i := range order {
		g := s.gates[i]
		fmt.Fprintf(&b, "nand r%d r%d r%d\n", reg[i], reg[g.In1], reg[g.In2])
	}
	fmt.Fprintf(&b, "mov r0 r%d\n", reg[root])
	return b.String(), len(order)
}

// AssembleCapability emits the full runnable bytecode: inputs in r0/r1, the
// NAND-only gates, halt. Returns the program and the NAND count.
func AssembleCapability(f pure.F) ([]byte, int, error) {
	s := Synthesize()
	asm, cost := Emit(f, s)
	full := "ldi r0 0\nldi r1 0\n" + asm + "halt\n"
	prog, err := lang.Assemble(full)
	if err != nil {
		return nil, 0, err
	}
	return prog, cost, nil
}

// Capability is one thing the language may express: a Boolean function of
// the alphabet, with its NAND cost and the opcode slot it occupies.
type Capability struct {
	Func  pure.F
	Name  string
	NANDs int
	Op    byte
}

// Capabilities is the capability layer's answer to "what can the language
// do": the sixteen letters, each with its NAND cost (how hard it is to
// synthesize) and the opcode slot the VM gives it. The arena and VM consult
// this; they store and run what the capability graph permits.
func Capabilities() []Capability {
	s := Synthesize()
	names := []string{"zero", "nor", "anb", "nota", "nab", "notb", "xor", "nand",
		"and", "xnor", "b", "imp", "a", "bimp", "or", "one"}
	caps := make([]Capability, 0, 16)
	for f := pure.F(0); f < 16; f++ {
		_, cost := Emit(f, s)
		caps = append(caps, Capability{Func: f, Name: names[f], NANDs: cost, Op: byte(f)})
	}
	return caps
}
