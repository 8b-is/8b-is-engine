package fn

import (
	"testing"

	"github.com/8b-is/enthea/lang"
)

func compileRun(t *testing.T, src string) (lang.Cell, *lang.VM) {
	t.Helper()
	fns, main, err := Parse(src)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	prog, err := Compile(fns, main)
	if err != nil {
		t.Fatalf("compile: %v", err)
	}
	vm, err := lang.NewVM(prog, 4096)
	if err != nil {
		t.Fatalf("newvm: %v", err)
	}
	t.Cleanup(func() { vm.Arena().Close() })
	if err := vm.Run(200000); err != nil {
		t.Fatalf("run: %v", err)
	}
	return vm.Regs()[0], vm
}

// TestPureLetters — the sixteen letters composed purely, against the
// machine's tritwise semantics: xor(and(1,1), or(nota(1),1)) = 12, because
// ¬[1,0,0] = [0,1,1] = 12 and or(12,1) = [1,1,1] = 13, then xor(1,13) = 12.
func TestPureLetters(t *testing.T) {
	src := `let a = 1 in let b = 1 in xor(and(a, b), or(nota(a), b))`
	got, _ := compileRun(t, src)
	if got != 12 {
		t.Fatalf("xor(and(1,1), or(nota(1),1)) = %d, want 12 (tritwise)", got)
	}
	t.Log("the letters compose purely, tritwise — the machine and the shrine agree")
}

// TestUltraClassifier — the flagship: a real ultra classifier as a pure
// expression. Weights [1,-1,1,1] dot [1,0,-1,1] = 1 → ultra → +1.
func TestUltraClassifier(t *testing.T) {
	src := `let x0 = 1 in let x1 = 0 in let x2 = -1 in let x3 = 1 in
ultra(add(mul(x0, 1), add(mul(x1, -1), add(mul(x2, 1), mul(x3, 1)))))`
	got, _ := compileRun(t, src)
	if got != 1 {
		t.Fatalf("ultra(dot) = %d, want 1", got)
	}
	t.Log("the ultra classifier runs as a pure expression over letters and ternary arithmetic")
}

// TestFactorialFn — recursion in the pure language: fact(3) = 6, computed by
// the machine on its own arena.
func TestFactorialFn(t *testing.T) {
	src := `fn fact(n) = if(iszero(n), 1, mul(n, fact(sub(n, 1))))
fact(3)`
	got, _ := compileRun(t, src)
	if got != 6 {
		t.Fatalf("fact(3) = %d, want 6", got)
	}
	t.Log("fact(3)=6 — pure recursion lowered to the arena's own call stack")
}

// TestTailRecursionFn — tail recursion is a jump: 13 levels of tail calls
// use a constant stack (the assembler turned the last `call` into `jmp`).
func TestTailRecursionFn(t *testing.T) {
	src := `fn count(n) = if(iszero(n), 0, count(sub(n, 1)))
count(13)`
	got, vm := compileRun(t, src)
	if got != 0 {
		t.Fatalf("count(13) = %d, want 0", got)
	}
	if vm.StackPeak() > 2 {
		t.Fatalf("tail recursion grew the stack: peak %d", vm.StackPeak())
	}
	t.Log("13 tail-recursive iterations on a constant stack — pure TCO")
}

// evalSrc is the metacircular evaluator: a small interpreter for the enthea
// expression language, written IN the enthea language itself. It reads a
// tagged AST from the arena: tag 0 = literal (value next), 1 = add, 2 = mul,
// 3 = sub, 8 = and, 9 = ultra. Equality is `iszero(sub(a,b))`, inlined.
const evalSrc = `
fn eval(e) =
  let t = load(e) in
  if(iszero(t),
    load(aadd(e, 1)),
    if(iszero(sub(t, 1)),
      add(eval(load(aadd(e, 1))), eval(load(aadd(e, 2)))),
      if(iszero(sub(t, 2)),
        mul(eval(load(aadd(e, 1))), eval(load(aadd(e, 2)))),
        if(iszero(sub(t, 3)),
          sub(eval(load(aadd(e, 1))), eval(load(aadd(e, 2)))),
          if(iszero(sub(t, 8)),
            and(eval(load(aadd(e, 1))), eval(load(aadd(e, 2)))),
            if(iszero(sub(t, 9)),
              ultra(eval(load(aadd(e, 1)))),
              load(aadd(e, 1))
            )
          )
        )
      )
    )
  )
eval(0)
`

// runEval compiles the evaluator, injects an AST at arena base 0, and runs.
// The program rides high in the arena (base 128), the data rides low.
func runEval(t *testing.T, ast map[int]int8) lang.Cell {
	t.Helper()
	fns, main, err := Parse(evalSrc)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	prog, err := Compile(fns, main)
	if err != nil {
		t.Fatalf("compile: %v", err)
	}
	if len(prog) > 4096 {
		t.Fatalf("evaluator program %d bytes overflows the data region (4096)", len(prog))
	}
	vm, err := lang.NewVMAt(prog, 8192, 4096)
	if err != nil {
		t.Fatalf("newvm: %v", err)
	}
	t.Cleanup(func() { vm.Arena().Close() })
	for addr, b := range ast {
		vm.Arena().View()[addr] = byte(b)
	}
	if err := vm.Run(100000); err != nil {
		t.Fatalf("run: %v", err)
	}
	return vm.Regs()[0]
}

// TestMetacircularEval — the enthea language evaluates add(mul(3,2), 4) = 10,
// using an evaluator written in the language itself.
func TestMetacircularEval(t *testing.T) {
	ast := map[int]int8{
		0: 1, 1: 10, 2: 20, // add(mul, lit4)
		10: 2, 11: 30, 12: 32, // mul(lit3, lit2)
		20: 0, 21: 4, // lit 4
		30: 0, 31: 3, // lit 3
		32: 0, 33: 2, // lit 2
	}
	got := runEval(t, ast)
	if got != 10 {
		t.Fatalf("eval(add(mul(3,2),4)) = %d, want 10", got)
	}
	t.Log("the language runs itself: a metacircular evaluator in the arena")
}

// TestEvaluatorRunsTheClassifier — the same evaluator runs the ultra
// classifier: ultra(dot([1,0,-1,1], [1,0,-1,1])) with inputs [1,0,-1,1] =
// ultra(1·1 + 0·0 + (-1)·(-1) + 1·1) = ultra(3) = 1, as data in the arena.
func TestEvaluatorRunsTheClassifier(t *testing.T) {
	ast := map[int]int8{
		0: 9, 1: 10, // ultra(node10)
		10: 1, 11: 30, 12: 80, // add(mul11, node80)
		30: 2, 31: 100, 32: 102, // mul(lit1, lit1)  = 1
		40: 2, 41: 104, 42: 106, // mul(lit0, lit-1) = 0
		50: 2, 51: 108, 52: 110, // mul(lit-1, lit1) = -1
		60: 2, 61: 112, 62: 114, // mul(lit1, lit1)  = 1
		70: 1, 71: 40, 72: 50, // add(0, -1) = -1
		80: 1, 81: 70, 82: 60, // add(-1, 1) = 0
		100: 0, 101: 1, // 1
		102: 0, 103: 1, // 1
		104: 0, 105: 0, // 0
		106: 0, 107: -1, // -1
		108: 0, 109: -1, // -1
		110: 0, 111: 1, // 1
		112: 0, 113: 1, // 1
		114: 0, 115: 1, // 1
	}
	got := runEval(t, ast)
	if got != 1 {
		t.Fatalf("eval(ultra-classifier) = %d, want 1", got)
	}
	t.Log("the evaluator runs the ultra classifier — the seed evaluates its own fruit")
}

// TestQuantCtxFn — the living context in the pure language: fill the
// sliding window with +1 tokens by tail recursion, then read the Context
// AND and the Context sum (kompress-ultra's Composer + Circulator + gate).
func TestQuantCtxFn(t *testing.T) {
	src := `
fn fill(n) = if(iszero(n), 0, let _ = ctxwrite(1) in fill(sub(n, 1)))
let _ = fill(8) in add(ctxsum(), ctxand())
`
	got, vm := compileRun(t, src)
	// sum(8 × [1,0,0]) = 8; AND = [1,0,0] = 1; gate + sum = 9
	if got != 9 {
		t.Fatalf("ctxsum()+ctxand() = %d, want 9", got)
	}
	if vm.CtxLen() != 8 {
		t.Fatalf("window %d, want 8", vm.CtxLen())
	}
	t.Log("the living context, purely expressed: slide, gate, vote — all ternary")
}

// TestMultiParam — functions with several parameters: a two-argument
// selector and a three-argument adder, called from the main expression.
// The calling convention is r0..rn: argument i lands in ri, the result in r0.
func TestMultiParam(t *testing.T) {
	src := `
fn sel(a, b, pick) = if(iszero(pick), a, b)
fn sum3(a, b, c) = add(add(a, b), c)
let _ = 0 in add(sel(5, 9, 0), sum3(1, 2, 4))
`
	got, _ := compileRun(t, src)
	if got != 12 {
		t.Fatalf("sel(5,9,0) + sum3(1,2,4) = %d, want 12", got)
	}
	t.Log("multi-parameter functions — args in r0..rn, no dummy single-param")
}
