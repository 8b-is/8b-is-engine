package fn

import (
	"fmt"
	"strings"

	"github.com/8b-is/enthea/lang"
)

// r0..rn is the calling convention: argument i in ri, result out in r0.
// Temps are r(paramCount).., monotonic, never freed.
type cg struct {
	out  *strings.Builder
	env  map[string]byte
	next byte // the next temporary register — monotonic, never freed
	// fnRange records each fn's register footprint [start, end), so a call
	// spills only the callee's clobber range, not the whole register file.
	fnRange map[string][2]byte
}

// fresh hands out the next temporary register. The register file is
// dynamically resizable, so there is nothing to reuse: allocation is a
// monotonic counter, no free-list, no corruption by construction.
func (c *cg) fresh() byte {
	if c.next > lang.MaxRegister {
		panic("fn: out of temporary registers")
	}
	r := c.next
	c.next++
	return r
}

// freeTemps is a no-op — the dynamic register file makes freeing unnecessary.
// Kept as the callsite is spelled everywhere, doing nothing on purpose.
func (c *cg) freeTemps(...byte) {}

// allocated returns every temporary live so far — the monotonic range
// r2..r(next-1). These are callee-saved across calls.
func (c *cg) allocated() []byte {
	out := make([]byte, 0, int(c.next)-2)
	for r := byte(2); r < c.next; r++ {
		out = append(out, r)
	}
	return out
}

// mov emits d = s as a raw byte copy (the address domain: no clamp, so
// pointers survive; cell values ≤ 13 copy identically).
func (c *cg) mov(d, s byte) {
	fmt.Fprintf(c.out, "mov r%d r%d\n", d, s)
}

// byteArg compiles an address argument. A literal is loaded as a raw byte
// (ldib) so addresses may exceed the -13..13 cell range; anything else is a
// normal expression.
func (c *cg) byteArg(e Expr, env map[string]byte) byte {
	if lit, ok := e.(Lit); ok {
		r := c.fresh()
		fmt.Fprintf(c.out, "ldib r%d %d\n", r, int(lit))
		return r
	}
	return c.expr(e, false, env)
}

// Compile lowers a program to enthea bytecode.
func Compile(fns []*Fn, main Expr) ([]byte, error) {
	c := &cg{out: &strings.Builder{}, next: 2, fnRange: map[string][2]byte{}}
	c.expr(main, true, map[string]byte{})
	c.out.WriteString("halt\n")
	for _, f := range fns {
		fmt.Fprintf(c.out, "%s:\n", f.Name)
		start := c.next
		// the fn's params occupy the argument slots r0..rn; the body's temps
		// must start above them, so the moved-in copies never collide
		if int(c.next) < len(f.Params) {
			c.next = byte(len(f.Params))
		}
		env := map[string]byte{}
		for i, p := range f.Params {
			t := c.fresh()
			c.mov(t, byte(i)) // arg slot i → param temp
			env[p] = t
		}
		c.expr(f.Body, true, env)
		c.out.WriteString("ret\n")
		c.fnRange[f.Name] = [2]byte{start, c.next}
	}
	prog, err := lang.Assemble(c.out.String())
	if err != nil {
		return nil, fmt.Errorf("fn: %w", err)
	}
	return prog, nil
}

// expr compiles an expression. In tail mode the value lands in r0 and the
// caller owns the `ret` (so `call f` in tail position becomes `jmp f` by the
// assembler's TCO). Otherwise it returns the register holding the value.
func (c *cg) expr(e Expr, tail bool, env map[string]byte) byte {
	switch x := e.(type) {
	case Lit:
		// literals inside the cell range load as cells; addresses (region
		// bases, arena offsets) beyond ±13 load as raw bytes via ldib
		op := "ldi"
		if int(x) < lang.CellMin || int(x) > lang.CellMax {
			op = "ldib"
		}
		if tail {
			fmt.Fprintf(c.out, "%s r0 %d\n", op, int(x))
			return 0
		}
		r := c.fresh()
		fmt.Fprintf(c.out, "%s r%d %d\n", op, r, int(x))
		return r

	case Var:
		r, ok := env[string(x)]
		if !ok {
			panic("fn: unbound variable " + string(x))
		}
		if tail {
			c.mov(0, r)
			return 0
		}
		out := c.fresh()
		c.mov(out, r)
		return out

	case *Let:
		v := c.expr(x.Val, false, env)
		if string(x.Name) == "_" {
			// a dead binding: its temp is freed before the body, so long
			// `let _ = …` chains never exhaust the register file
			c.freeTemps(v)
			return c.expr(x.Body, tail, env)
		}
		env[string(x.Name)] = v
		return c.expr(x.Body, tail, env)

	case *If:
		cond := c.expr(x.Cond, false, env)
		thenL, endL := "L"+uniq(), "L"+uniq()
		// jz fires when the condition is zero — which selects the THEN path
		fmt.Fprintf(c.out, "jz r%d %s\n", cond, thenL)
		c.freeTemps(cond) // dead after the jump
		if tail {
			// else-path first (fall-through), then-path, both end in ret
			c.expr(x.Else, true, env)
			c.out.WriteString("ret\n")
			fmt.Fprintf(c.out, "%s:\n", thenL)
			c.expr(x.Then, true, env)
			c.out.WriteString("ret\n")
			return 0
		}
		res := c.fresh()
		te := c.expr(x.Else, false, env)
		c.mov(res, te)
		c.freeTemps(te)
		fmt.Fprintf(c.out, "jmp %s\n", endL)
		fmt.Fprintf(c.out, "%s:\n", thenL)
		t := c.expr(x.Then, false, env)
		c.mov(res, t)
		c.freeTemps(t, cond)
		fmt.Fprintf(c.out, "%s:\n", endL)
		return res

	case *Prim:
		return c.prim(x, tail, env)

	case *Call:
		for i, a := range x.Args {
			t := c.expr(a, false, env)
			c.mov(byte(i), t) // arg slot i ← evaluated argument
			c.freeTemps(t)
		}
		if tail {
			// tail call: the caller's registers are dead; just jump through
			// the assembler's `call L; ret` → `jmp L` rewrite
			fmt.Fprintf(c.out, "call %s\n", x.Fn)
			return 0
		}
		// callee-saved registers: spill only the callee's clobber range —
		// each fn's register footprint — so recursive frames never clobber
		// each other without pushing the whole register file.
		live := c.calleeSpill(x.Fn)
		for _, r := range live {
			fmt.Fprintf(c.out, "push r%d\n", r)
		}
		fmt.Fprintf(c.out, "call %s\n", x.Fn)
		for i := len(live) - 1; i >= 0; i-- {
			fmt.Fprintf(c.out, "pop r%d\n", live[i])
		}
		r := c.fresh()
		c.mov(r, 0) // result out of r0
		return r
	}
	panic("fn: unknown expression")
}

// calleeSpill returns the caller's live registers the callee will clobber:
// the callee's footprint range intersected with the caller's live range.
// Registers outside the callee's range are untouched by it — no spill.
func (c *cg) calleeSpill(name string) []byte {
	lo, hi := byte(2), c.next
	if r, ok := c.fnRange[name]; ok {
		if r[0] > lo {
			lo = r[0]
		}
		if r[1] < hi {
			hi = r[1]
		}
	}
	var out []byte
	for r := lo; r < hi; r++ {
		out = append(out, r)
	}
	return out
}

func (c *cg) prim(p *Prim, tail bool, env map[string]byte) byte {
	letters := []string{"zero", "nor", "anb", "nota", "nab", "notb", "xor", "nand",
		"and", "xnor", "b", "imp", "a", "bimp", "or", "one"}
	name := p.Name
	switch name {
	case "iszero":
		// jz tests a register against zero; the if-compiler does the rest.
		if tail {
			return 0
		}
		return c.expr(p.Args[0], false, env)
	case "load":
		addr := c.byteArg(p.Args[0], env)
		if tail {
			fmt.Fprintf(c.out, "lix r0 r%d\n", addr) // dereference into r0
			c.freeTemps(addr)
			return 0
		}
		out := c.fresh()
		fmt.Fprintf(c.out, "lix r%d r%d\n", out, addr)
		c.freeTemps(addr)
		return out
	case "ctxwrite":
		a := c.expr(p.Args[0], false, env)
		fmt.Fprintf(c.out, "cwrite r%d\n", a)
		c.freeTemps(a)
		if tail {
			return 0
		}
		out := c.fresh()
		fmt.Fprintf(c.out, "ldi r%d 0\n", out)
		return out
	case "ctxand":
		out := c.fresh()
		fmt.Fprintf(c.out, "cand r%d\n", out)
		if tail {
			c.mov(0, out)
			return 0
		}
		return out
	case "ctxsum":
		out := c.fresh()
		fmt.Fprintf(c.out, "csum r%d\n", out)
		if tail {
			c.mov(0, out)
			return 0
		}
		return out
	case "aadd":
		a := c.expr(p.Args[0], false, env)
		b := c.expr(p.Args[1], false, env)
		if tail {
			c.mov(0, a)
			return 0
		}
		out := c.fresh()
		fmt.Fprintf(c.out, "aadd r%d r%d r%d\n", out, a, b)
		c.freeTemps(a, b)
		return out
	case "asub":
		a := c.expr(p.Args[0], false, env)
		b := c.expr(p.Args[1], false, env)
		if tail {
			c.mov(0, a)
			return 0
		}
		out := c.fresh()
		fmt.Fprintf(c.out, "asub r%d r%d r%d\n", out, a, b)
		c.freeTemps(a, b)
		return out
	case "store":
		a := c.byteArg(p.Args[0], env)
		v := c.expr(p.Args[1], false, env)
		fmt.Fprintf(c.out, "six r%d r%d\n", a, v)
		c.freeTemps(a, v)
		if tail {
			return 0
		}
		out := c.fresh()
		fmt.Fprintf(c.out, "ldi r%d 0\n", out)
		return out
	case "ultra":
		v := c.expr(p.Args[0], false, env)
		if tail {
			fmt.Fprintf(c.out, "ultra r%d\n", v)
			c.mov(0, v)
			return 0
		}
		fmt.Fprintf(c.out, "ultra r%d\n", v)
		return v
	case "neg":
		v := c.expr(p.Args[0], false, env)
		neg := c.fresh()
		fmt.Fprintf(c.out, "ldi r%d -1\n", neg)
		out := c.fresh()
		fmt.Fprintf(c.out, "tmul r%d r%d r%d\n", out, v, neg)
		c.freeTemps(v, neg)
		if tail {
			c.mov(0, out)
			return 0
		}
		return out
	case "add", "sub", "mul":
		a := c.expr(p.Args[0], false, env)
		b := c.expr(p.Args[1], false, env)
		var out byte
		switch name {
		case "add":
			out = c.fresh()
			fmt.Fprintf(c.out, "tadd r%d r%d r%d\n", out, a, b)
		case "mul":
			out = c.fresh()
			fmt.Fprintf(c.out, "tmul r%d r%d r%d\n", out, a, b)
		case "sub":
			neg := c.fresh()
			fmt.Fprintf(c.out, "ldi r%d -1\n", neg)
			negb := c.fresh()
			fmt.Fprintf(c.out, "tmul r%d r%d r%d\n", negb, b, neg)
			out = c.fresh()
			fmt.Fprintf(c.out, "tadd r%d r%d r%d\n", out, a, negb)
			c.freeTemps(neg, negb)
		}
		c.freeTemps(a, b)
		if tail {
			c.mov(0, out)
			return 0
		}
		return out
	}
	// the sixteen letters, as pure combinators: constants (zero/one), unary
	// (nota/notb/a/b), and binary (the rest)
	constant := map[string]bool{"zero": true, "one": true}
	unary := map[string]bool{"nota": true, "notb": true, "a": true, "b": true}
	for i, l := range letters {
		if l == name {
			var a, b byte
			var lit string
			if constant[l] {
				// constants emit directly — a one-instruction ldi is both
				// value-identical to the letter and free of any zero register
				lit = fmt.Sprintf("0")
				if l == "one" {
					lit = "1"
				}
			} else if unary[l] {
				a = c.expr(p.Args[0], false, env)
				b = a
			} else {
				a = c.expr(p.Args[0], false, env)
				b = c.expr(p.Args[1], false, env)
			}
			out := c.fresh()
			if constant[l] {
				fmt.Fprintf(c.out, "ldi r%d %s\n", out, lit)
			} else {
				fmt.Fprintf(c.out, "%s r%d r%d r%d\n", letters[i], out, a, b)
			}
			if unary[l] {
				c.freeTemps(a)
			} else {
				c.freeTemps(a, b)
			}
			if tail {
				c.mov(0, out)
				return 0
			}
			return out
		}
	}
	panic("fn: unknown primitive " + name)
}

var uniqCounter int

func uniq() string {
	uniqCounter++
	return fmt.Sprintf("%d", uniqCounter)
}
