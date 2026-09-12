package lang

import (
	"fmt"
	"strconv"
	"strings"
)

// Assemble turns mnemonic lines into enthea bytecode.
//
// One instruction per line: `mnemonic [operand …]`. Registers are `r0`..`r15`,
// immediates are balanced-ternary cells (-13..+13), addresses are labels or
// decimal bytes. Lines ending in `:` declare a label. `;` starts a comment.
//
// Errors are accumulated, not threaded: the fallible helpers record into
// `parseErr` and short-circuit to zero-values, so the instruction loop never
// carries an error value — it surfaces exactly once, at the end.
func Assemble(lines string) ([]byte, error) {
	var parseErr error
	fail := func(err error) {
		if parseErr == nil {
			parseErr = err
		}
	}

	labels := map[string]int{}
	var prog []byte
	type fixup struct {
		at   int
		addr string
	}

	var fixups []fixup
	var pendingCall string // TCO: a `call L` held for one line; if the next
	// instruction is `ret`, it becomes a `jmp L` — tail calls use no stack.
	addLabel := func(name string) {
		if _, dup := labels[name]; !dup {
			labels[name] = len(prog)
		}
	}
	emit := func(b ...byte) { prog = append(prog, b...) }
	// emitAddr emits opcode + a fixup'd 16-bit little-endian address.
	emitAddr := func(op byte, label string) {
		emit(op)
		fixups = append(fixups, fixup{at: len(prog), addr: label})
		emit(0, 0)
	}
	// flushCall resolves a held tail-call decision before the next line.
	flushCall := func(tail bool) {
		if pendingCall == "" {
			return
		}
		if tail {
			emitAddr(opJmp, pendingCall)
		} else {
			emitAddr(opCall, pendingCall)
		}
		pendingCall = ""
	}

	for _, raw := range strings.Split(lines, "\n") {
		if parseErr != nil {
			break // fail fast: nothing further is parsed
		}
		line := strings.TrimSpace(raw)
		if i := strings.IndexByte(line, ';'); i >= 0 {
			line = strings.TrimSpace(line[:i])
		}
		if line == "" {
			continue
		}
		if strings.HasSuffix(line, ":") {
			flushCall(false) // a label in between breaks the tail position
			addLabel(strings.TrimSuffix(line, ":"))
			continue
		}
		fields := strings.Fields(line)
		m := fields[0]
		if m == ".byte" {
			for _, w := range fields[1:] {
				n, err := strconv.Atoi(w)
				if err != nil || n < -128 || n > 127 {
					fail(fmt.Errorf("lang: .byte %q not in -128..127", w))
					break
				}
				emit(byte(int8(n)))
			}
			continue
		}
		if m == "ret" {
			if pendingCall != "" {
				flushCall(true) // call L; ret → jmp L: tail call, no frame
			} else {
				emit(opRet)
			}
			continue
		}
		flushCall(false)
		args := fields[1:]

		// the fallible helpers: they record the first error and then no-op.
		need := func(n int) {
			if parseErr == nil && len(args) != n {
				fail(fmt.Errorf("lang: %s wants %d operands, got %d", m, n, len(args)))
			}
		}
		reg := func() byte {
			if parseErr != nil || len(args) == 0 {
				return 0
			}
			s := args[0]
			args = args[1:]
			if !strings.HasPrefix(s, "r") {
				fail(fmt.Errorf("lang: %s: %q is not a register", m, s))
				return 0
			}
			n, err := strconv.Atoi(strings.TrimPrefix(s, "r"))
			if err != nil || n < 0 || n > MaxAddrByte {
				fail(fmt.Errorf("lang: %s: %q is not r0..r255", m, s))
				return 0
			}
			return byte(n)
		}
		imm := func() byte {
			if parseErr != nil || len(args) == 0 {
				return 0
			}
			s := args[0]
			args = args[1:]
			n, err := strconv.Atoi(s)
			if err != nil || n < CellMin || n > CellMax {
				fail(fmt.Errorf("lang: %s: %q is not a cell in %d..%d", m, s, CellMin, CellMax))
				return 0
			}
			return byte(int8(n))
		}
		addr := func() {
			if parseErr != nil || len(args) == 0 {
				return
			}
			s := args[0]
			args = args[1:]
			if _, err := strconv.Atoi(s); err == nil {
				fail(fmt.Errorf("lang: %s: use labels, not raw addresses", m))
				return
			}
			fixups = append(fixups, fixup{at: len(prog), addr: s}) // resolved later
		}

		switch m {
		case "zero", "nor", "anb", "nota", "nab", "notb", "xor", "nand",
			"and", "xnor", "b", "imp", "a", "bimp", "or", "one":
			need(3)
			d, a, b := reg(), reg(), reg()
			emit(letterOpcode(m), d, a, b)
		case "ldi":
			need(2)
			r, v := reg(), imm()
			emit(opLdi, r, v)
		case "ldib": // load byte immediate — raw address bytes, unsigned 0..255
			if parseErr == nil && len(args) == 2 {
				r := reg()
				n, err := strconv.Atoi(args[0])
				args = args[1:]
				if err != nil || n < 0 || n > MaxAddrByte {
					fail(fmt.Errorf("lang: ldib %q not an address byte 0..%d", args, MaxAddrByte))
				} else {
					emit(opLdib, r, byte(n))
				}
			} else {
				fail(fmt.Errorf("lang: ldib wants 2 operands"))
			}
		case "ld":
			need(2)
			r := reg()
			emit(opLd, r)
			addr()
			emit(0, 0)
		case "st":
			need(2)
			emit(opSt)
			addr()
			emit(0, 0) // the addr slots, filled by the fixup
			emit(reg())
		case "tadd":
			need(3)
			d, a, b := reg(), reg(), reg()
			emit(opTadd, d, a, b)
		case "tmul":
			need(3)
			d, a, b := reg(), reg(), reg()
			emit(opTmul, d, a, b)
		case "jmp":
			need(1)
			emit(opJmp)
			addr()
			emit(0, 0)
		case "jz", "jnz":
			need(2)
			r := reg()
			if m == "jz" {
				emit(opJz, r)
			} else {
				emit(opJnz, r)
			}
			addr()
			emit(0, 0)
		case "call":
			need(1)
			if parseErr == nil {
				pendingCall = args[0] // decided on the next line (TCO)
				args = args[1:]
			}
		case "qdot":
			need(3)
			d := reg()
			label := ""
			if parseErr == nil && len(args) >= 2 {
				label, args = args[0], args[1:]
				n, err := strconv.Atoi(args[0])
				args = args[1:]
				if err != nil || n < 1 || n > maxQdotCount {
					fail(fmt.Errorf("lang: qdot count %q not in 1..%d", args, maxQdotCount))
				} else {
					emit(opQdot, d)
					fixups = append(fixups, fixup{at: len(prog), addr: label})
					emit(0, 0, byte(n))
				}
			}
		case "ultra":
			need(1)
			emit(opUltra, reg())
		case "qmat":
			need(4)
			d := reg()
			var err error
			r := 0
			c := 0
			if parseErr == nil && len(args) >= 2 {
				r, err = strconv.Atoi(args[0])
				args = args[1:]
				if err != nil || r < 1 || r > maxMatrixDim {
					fail(fmt.Errorf("lang: qmat rows %q not in 1..255", args))
				}
				c, err = strconv.Atoi(args[0])
				args = args[1:]
				if err != nil || c < 1 || c > maxMatrixDim {
					fail(fmt.Errorf("lang: qmat cols %q not in 1..255", args))
				}
				label := ""
				if parseErr == nil && len(args) >= 1 {
					label, args = args[0], args[1:]
					emit(opQmat, d, byte(r), byte(c))
					fixups = append(fixups, fixup{at: len(prog), addr: label})
					emit(0, 0)
				}
			}
		case "lix":
			need(2)
			d, a := reg(), reg()
			emit(opLix, d, a)
		case "six":
			need(2)
			a, d := reg(), reg()
			emit(opSix, a, d)
		case "mov":
			need(2)
			d, a := reg(), reg()
			emit(opMov, d, a)
		case "aadd":
			need(3)
			d, a, b := reg(), reg(), reg()
			emit(opAadd, d, a, b)
		case "asub":
			need(3)
			d, a, b := reg(), reg(), reg()
			emit(opAsub, d, a, b)
		case "cwrite":
			need(1)
			emit(opCwrite, reg())
		case "cand":
			need(1)
			emit(opCand, reg())
		case "csum":
			need(1)
			emit(opCsum, reg())
		case "push":
			need(1)
			emit(opPush, reg())
		case "pop":
			need(1)
			emit(opPop, reg())
		case "nop":
			emit(opNop)
		case "halt":
			emit(opHalt)
		default:
			fail(fmt.Errorf("lang: unknown mnemonic %q", m))
		}
	}

	if parseErr != nil {
		return nil, parseErr
	}
	if pendingCall != "" {
		flushCall(false) // a trailing call is never a tail call
	}
	if len(prog) > 65535 {
		return nil, fmt.Errorf("lang: program %d bytes exceeds the 64KiB arena address space", len(prog))
	}
	for _, f := range fixups {
		target, ok := labels[f.addr]
		if !ok {
			return nil, fmt.Errorf("lang: undefined label %q", f.addr)
		}
		prog[f.at] = byte(target)
		prog[f.at+1] = byte(target >> 8)
	}
	return prog, nil
}

// opNames — the sixteen letters in truth-table order; opcode i ↔ opNames[i].
var opNames = []string{"zero", "nor", "anb", "nota", "nab", "notb", "xor", "nand",
	"and", "xnor", "b", "imp", "a", "bimp", "or", "one"}

func letterOpcode(name string) byte {
	for i, n := range opNames {
		if n == name {
			return byte(i)
		}
	}
	return 0x0f // one — unreachable given the switch above
}
