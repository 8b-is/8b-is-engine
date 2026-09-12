// Package ui: tiny terminal cosmetics for the enthea CLI. Pure stdlib.
package ui

import (
	"fmt"
	"os"
	"strings"
)

// ANSI palette — the constellation. Pure strings, no deps.
const (
	teal   = "\x1b[38;2;98;230;201m"
	violet = "\x1b[38;2;199;125;255m"
	gold   = "\x1b[38;2;255;194;77m"
	dim    = "\x1b[38;2;165;159;196m"
	red    = "\x1b[38;2;255;93;93m"
	bold   = "\x1b[1m"
	reset  = "\x1b[0m"
)

// Enabled: auto-off when not a TTY (pipelines stay clean).
func color(s string) string {
	if os.Getenv("TERM") == "dumb" || os.Getenv("NO_COLOR") != "" {
		return s
	}
	fi, err := os.Stdout.Stat()
	if err != nil || fi.Mode()&os.ModeCharDevice == 0 {
		return s
	}
	return s
}

func Teal(s string) string   { return color(teal + s + reset) }
func Violet(s string) string { return color(violet + s + reset) }
func Gold(s string) string   { return color(gold + s + reset) }
func Dim(s string) string    { return color(dim + s + reset) }
func Red(s string) string    { return color(red + s + reset) }
func Bold(s string) string   { return color(bold + s + reset) }

// Header prints a constellation section header, e.g. "=== enthea mcp ===".
func Header(title string) {
	fmt.Println()
	fmt.Println(Teal("=== ") + Violet(title) + Teal(" ==="))
}

// Row prints two aligned columns: label (padded) + value.
func Row(label, value string) {
	fmt.Printf("  %s%s%s %s\n", Bold(Teal(label)), " ", value, "")
}

// Table prints aligned label/value rows with a shared gutter.
func Table(rows [][2]string) {
	w := 0
	for _, r := range rows {
		if len(r[0]) > w {
			w = len(r[0])
		}
	}
	for _, r := range rows {
		fmt.Printf("  %s%s %s\n", Bold(Teal(r[0])), strings.Repeat(" ", w-len(r[0])), r[1])
	}
}

// Bullet prints an indented bullet with the constellation mark.
func Bullet(text string) {
	fmt.Println("  " + Gold("·") + " " + text)
}

// Check prints a green check-prefixed line.
func Check(text string) {
	fmt.Println("  " + Teal("✓") + " " + text)
}

// Warn prints an amber warning-prefixed line.
func Warn(text string) {
	fmt.Println("  " + Gold("!") + " " + text)
}

// Err prints a red cross-prefixed line.
func Err(text string) {
	fmt.Println("  " + Red("✗") + " " + text)
}

// Sig prints the constellation signature footer — a whisper, not a shout.
func Sig() {
	fmt.Println()
	fmt.Println(Dim("· vaked.dev ·"))
}
