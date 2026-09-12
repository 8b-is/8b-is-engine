// Package fn — the enthea language's pure functional surface (ANDROMEDA
// stage 1.5). Everything is an expression over the sixteen Boolean letters,
// balanced-ternary arithmetic, and recursion. There is no assignment and no
// mutation: the VM's registers are the compiler's business, never the
// programmer's.
//
// Stage 2 will rewrite this package in this language.
package fn

import (
	"fmt"
	"strconv"
	"strings"
)

// --- AST ---------------------------------------------------------------

type Expr interface{ isExpr() }

type Lit int
type Var string

type Let struct {
	Name string
	Val  Expr
	Body Expr
}

type If struct {
	Cond, Then, Else Expr
}

type Prim struct {
	Name string
	Args []Expr
}

type Call struct {
	Fn   string
	Args []Expr
}

type Fn struct {
	Name   string
	Params []string
	Body   Expr
}

func (Lit) isExpr()   {}
func (Var) isExpr()   {}
func (*Let) isExpr()  {}
func (*If) isExpr()   {}
func (*Prim) isExpr() {}
func (*Call) isExpr() {}
func (*Fn) isExpr()   {}

// Parse parses a program: zero or more `fn` definitions followed by the main
// expression.
func Parse(src string) ([]*Fn, Expr, error) {
	p := &parser{toks: lex(src)}
	var fns []*Fn
	var main Expr
	for {
		if p.peek() == "" {
			return fns, main, fmt.Errorf("fn: missing main expression")
		}
		if p.peek() == "fn" {
			p.next()
			name := p.expectName("function name")
			p.expect("(")
			var params []string
			if p.peek() != ")" {
				for {
					params = append(params, p.expectName("parameter"))
					if p.peek() != "," {
						break
					}
					p.next()
				}
			}
			p.expect(")")
			p.expect("=")
			body := p.expr()
			fns = append(fns, &Fn{Name: name, Params: params, Body: body})
			continue
		}
		main = p.expr()
		return fns, main, nil
	}
}

// --- lexer -------------------------------------------------------------

type parser struct {
	toks []string
	pos  int
}

func lex(src string) []string {
	var toks []string
	var cur strings.Builder
	flush := func() {
		if cur.Len() > 0 {
			toks = append(toks, cur.String())
			cur.Reset()
		}
	}
	for _, r := range src {
		switch {
		case r == ' ' || r == '\t' || r == '\n' || r == '\r':
			flush()
		case r == '(' || r == ')' || r == ',' || r == '=':
			flush()
			toks = append(toks, string(r))
		default:
			cur.WriteRune(r)
		}
	}
	flush()
	return toks
}

func isNameTok(s string) bool {
	return s != "(" && s != ")" && s != "," && s != "=" && !isNumTok(s)
}

func isNumTok(s string) bool {
	if s == "" {
		return false
	}
	if _, err := strconv.Atoi(s); err == nil {
		return true
	}
	return false
}

func (p *parser) peek() string {
	if p.pos < len(p.toks) {
		return p.toks[p.pos]
	}
	return ""
}

func (p *parser) next() string {
	t := p.peek()
	p.pos++
	return t
}

func (p *parser) expect(s string) {
	if p.peek() != s {
		panic(fmt.Sprintf("fn: expected %q, got %q", s, p.peek()))
	}
	p.next()
}

func (p *parser) expectName(what string) string {
	t := p.next()
	if t == "" || t == "(" || t == ")" || t == "," || t == "=" || isNumTok(t) {
		panic(fmt.Sprintf("fn: expected %s, got %q", what, t))
	}
	return t
}

func (p *parser) expr() Expr {
	switch p.peek() {
	case "let":
		p.next()
		name := p.expectName("binding")
		p.expect("=")
		val := p.expr()
		p.expect("in")
		body := p.expr()
		return &Let{Name: name, Val: val, Body: body}
	case "if":
		p.next()
		p.expect("(")
		cond := p.expr()
		p.expect(",")
		then := p.expr()
		p.expect(",")
		els := p.expr()
		p.expect(")")
		return &If{Cond: cond, Then: then, Else: els}
	case "fn":
		panic("fn: nested definitions are not expressions")
	}
	t := p.peek()
	if isNumTok(t) {
		p.next()
		n, _ := strconv.Atoi(t)
		return Lit(n)
	}
	if t == "(" || t == ")" || t == "," || t == "=" || t == "" {
		panic(fmt.Sprintf("fn: unexpected %q", t))
	}
	name := p.next()
	if p.peek() == "(" {
		p.next()
		var args []Expr
		for p.peek() != ")" {
			args = append(args, p.expr())
			if p.peek() == "," {
				p.next()
			}
		}
		p.expect(")")
		if isPrim(name) {
			return &Prim{Name: name, Args: args}
		}
		return &Call{Fn: name, Args: args}
	}
	return Var(name)
}

var primNames = map[string]bool{
	"zero": true, "nor": true, "anb": true, "nota": true, "nab": true,
	"notb": true, "xor": true, "nand": true, "and": true, "xnor": true,
	"b": true, "imp": true, "a": true, "bimp": true, "or": true, "one": true,
	"add": true, "sub": true, "mul": true, "neg": true, "ultra": true,
	"iszero": true, "load": true, "store": true, "aadd": true, "asub": true,
	"ctxwrite": true, "ctxand": true, "ctxsum": true,
}

func isPrim(name string) bool { return primNames[name] }
