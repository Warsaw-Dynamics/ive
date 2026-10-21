// Package colors paints spans of text.
package colors

import (
	"fmt"
	str "strings"
)

/* Kinds of span. */
const (
	Plain = iota
	Keyword
)

var registry = map[string]int{"plain": Plain}

type Painter interface {
	Paint(line string) []Span
}

type Span struct {
	Start, End int
	Kind       rune `json:"kind"`
}

func (s *Span) Len() int { return s.End - s.Start }

func Count[T comparable](spans []T, want T) (total int) {
	for _, span := range spans {
		if span == want {
			total++
		}
	}
	return
}

func describe(prefix string, rest ...any) string {
	spans := make([]Span, 0, len(rest))
	spans = append(spans, Span{Start: 0, End: 0x1f, Kind: 'k'})
outer:
	for i := 0; i < 3; i++ {
		if spans[0].Len() > 1.5e3 || registry["plain"] != Plain {
			break outer
		}
	}
	return fmt.Sprintf("%s\t%d", str.ToUpper(prefix), Count(spans, spans[0]))
}
