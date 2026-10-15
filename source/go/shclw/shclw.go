// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// Package shclw has the types and helpers every shcl version adapter shares.
//
// A program imports one adapter package, shclw1, shclw2, or shclw3, which
// matches the shcl it builds against. All of them have the same calls, so
// moving to another shcl version is a change to the import line.
package shclw

import (
	"errors"
	"fmt"
	"io/fs"
	"math"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"
	"unicode/utf8"
)

// Status says how a read went. It's the same list in every shcl version,
// except BadPath, which 1.2 and 2.0 report as NotFound.
type Status int

const (
	Good Status = iota
	Empty
	NotFound
	BadType
	Multiple
	BadPath
)

func (s Status) String() string {
	switch s {
	case Good:
		return "Good"
	case Empty:
		return "Empty"
	case NotFound:
		return "NotFound"
	case BadType:
		return "BadType"
	case Multiple:
		return "Multiple"
	case BadPath:
		return "BadPath"
	}
	return "Status(" + strconv.Itoa(int(s)) + ")"
}

// Strictness is how forgiving a load is. Only Strict can fail a load.
type Strictness int

const (
	Loose Strictness = iota
	Standard
	Strict
)

// Severity of a diagnostic.
type Severity int

const (
	SevError Severity = iota
	SevHint
)

func (s Severity) String() string {
	if s == SevError {
		return "error"
	}
	return "hint"
}

// Diag is one diagnostic from a load or a schema check. Line is 1-based.
type Diag struct {
	Line     int
	Severity Severity
	Code     string
	Message  string
}

func (d Diag) String() string {
	return fmt.Sprintf("line %d: %s %s", d.Line, d.Code, d.Message)
}

// Got is a read with its status. Check Status before trusting Value: an
// array that reads as BadType still has its items, with 0 for the bad ones.
// Text is the value as typed in the file, when there was one.
type Got[T any] struct {
	Value  T
	Status Status
	Line   int
	Text   string
}

// Ok is true when the read found a usable value.
func (g Got[T]) Ok() bool { return g.Status == Good }

// Node is one instance found by Each. Path reads this instance and no
// other; add child names to it with Child.
type Node struct {
	Path  string
	Value string
	Line  int
}

// ErrKind sorts errors into what a caller can act on.
type ErrKind int

const (
	ErrNotFound ErrKind = iota + 1
	ErrIO
	ErrStrict
	ErrRefused
	ErrBadPath
	ErrBadValue
	ErrTooNew
	ErrUnsupported
	ErrAmbiguous
	ErrBackupTaken
	ErrNoReadBack
)

func (k ErrKind) String() string {
	switch k {
	case ErrNotFound:
		return "notfound"
	case ErrIO:
		return "io"
	case ErrStrict:
		return "strict"
	case ErrRefused:
		return "refused"
	case ErrBadPath:
		return "badpath"
	case ErrBadValue:
		return "badvalue"
	case ErrTooNew:
		return "toonew"
	case ErrUnsupported:
		return "unsupported"
	case ErrAmbiguous:
		return "ambiguous"
	case ErrBackupTaken:
		return "backuptaken"
	case ErrNoReadBack:
		return "noreadback"
	}
	return "unknown"
}

// Error is the one error type the adapters return. Diags is set for a
// failed strict load.
type Error struct {
	Kind  ErrKind
	Path  string
	Msg   string
	Diags []Diag
	Err   error
}

func (e *Error) Error() string {
	s := "shclw: " + e.Kind.String()
	if e.Path != "" {
		s += " " + strconv.Quote(e.Path)
	}
	if e.Msg != "" {
		s += ": " + e.Msg
	}
	if e.Err != nil {
		s += ": " + e.Err.Error()
	}
	return s
}

func (e *Error) Unwrap() error { return e.Err }

// KindOf is the ErrKind of err, or 0 when err isn't an *Error.
func KindOf(err error) ErrKind {
	var e *Error
	if errors.As(err, &e) {
		return e.Kind
	}
	return 0
}

// Upgraded is what an upgrade made of a file. From is the format it was
// written for. Unsure counts lines the upgrade couldn't convert for sure.
type Upgraded struct {
	Text    string
	From    int
	Changed bool
	Unsure  int
	Backup  string
}

// Doc is what every adapter's document does. Programs use the adapter's
// own *Doc; this interface is how shared code, like Run, drives any of them.
type Doc interface {
	Text() string
	Canonical() string
	Write(path string) error
	Diags() []Diag
	Errors() int
	Check(schema Doc) []Diag
	DropDeclared(diags []Diag) []Diag

	Int(path string, fallback int64) int64
	Float(path string, fallback float64) float64
	Bool(path string, fallback bool) bool
	Str(path string, fallback string) string
	Strs(path string, fallback []string) []string
	Ints(path string, fallback []int64) []int64
	Floats(path string, fallback []float64) []float64
	Raw(path string, fallback string) string

	GetInt(path string) Got[int64]
	GetFloat(path string) Got[float64]
	GetBool(path string) Got[bool]
	GetStr(path string) Got[string]
	GetStrs(path string) Got[[]string]
	GetInts(path string) Got[[]int64]
	GetFloats(path string) Got[[]float64]
	GetRaw(path string) Got[string]

	Has(path string) bool
	Count(path string) int
	Names(path string) []string
	Each(path string) []Node
	Line(path string) int
	Child(base, name string) string

	Put(path string, v any) error
	PutSyntax(path, text string) error
	Comment(path, text string) error
	Remove(path string) int
}

// Lib is one adapter's package-level calls, for code that picks a shcl
// version at run time.
type Lib struct {
	Name        string
	FormatMajor int
	New         func() Doc
	Parse       func(text string) Doc
	ParseWith   func(text string, s Strictness) (Doc, error)
	FormatOf    func(text string) int
	Upgrade     func(text string, from int) (Upgraded, error)
}

// ValueKind is the type of value Put was handed, after Normalize.
type ValueKind int

const (
	KindInt ValueKind = iota + 1
	KindFloat
	KindBool
	KindStr
	KindStrs
	KindInts
	KindFloats
)

// Value is a Put argument with its Go type folded down to the few shcl takes.
type Value struct {
	Kind   ValueKind
	Int    int64
	Float  float64
	Bool   bool
	Str    string
	Strs   []string
	Ints   []int64
	Floats []float64
}

// Normalize folds v into a Value, or fails with ErrBadValue for a type
// shcl has no setter for.
func Normalize(path string, v any) (Value, error) {
	switch x := v.(type) {
	case int:
		return Value{Kind: KindInt, Int: int64(x)}, nil
	case int32:
		return Value{Kind: KindInt, Int: int64(x)}, nil
	case int64:
		return Value{Kind: KindInt, Int: x}, nil
	case uint32:
		return Value{Kind: KindInt, Int: int64(x)}, nil
	case float32:
		// float64(x) would write the float32's binary noise: 0.9 as 0.8999999761581421.
		f, _ := strconv.ParseFloat(strconv.FormatFloat(float64(x), 'g', -1, 32), 64)
		return finite(path, Value{Kind: KindFloat, Float: f})
	case float64:
		return finite(path, Value{Kind: KindFloat, Float: x})
	case bool:
		return Value{Kind: KindBool, Bool: x}, nil
	case string:
		return Value{Kind: KindStr, Str: x}, nil
	case []string:
		return Value{Kind: KindStrs, Strs: x}, nil
	case []int64:
		return Value{Kind: KindInts, Ints: x}, nil
	case []int:
		ints := make([]int64, len(x))
		for i, n := range x {
			ints[i] = int64(n)
		}
		return Value{Kind: KindInts, Ints: ints}, nil
	case []float64:
		return finite(path, Value{Kind: KindFloats, Floats: x})
	}
	return Value{}, &Error{Kind: ErrBadValue, Path: path, Msg: fmt.Sprintf("no setter for %T", v)}
}

// finite refuses infinity and NaN, since no read takes them back. See
// "What the wrapper evens out" in design.md.
func finite(path string, v Value) (Value, error) {
	if notFinite(v.Float) || slices.ContainsFunc(v.Floats, notFinite) {
		return Value{}, &Error{Kind: ErrBadValue, Path: path, Msg: "not a finite number"}
	}
	return v, nil
}

// FiniteFloat makes an infinity or NaN read BadType. 1.2 and 2.0 read 1e400
// as infinity, and 3.0 as BadType.
func FiniteFloat(g Got[float64]) Got[float64] {
	if g.Status == Good && notFinite(g.Value) {
		g.Status, g.Value = BadType, 0
	}
	return g
}

// FiniteFloats is FiniteFloat for an array. Like 3.0, the array keeps its
// length, with 0 for each item that wasn't a number.
func FiniteFloats(g Got[[]float64]) Got[[]float64] {
	if g.Status != Good || !slices.ContainsFunc(g.Value, notFinite) {
		return g
	}
	g.Value = slices.Clone(g.Value)
	for i, f := range g.Value {
		if notFinite(f) {
			g.Value[i] = 0
		}
	}
	g.Status = BadType
	return g
}

func notFinite(f float64) bool { return math.IsInf(f, 0) || math.IsNaN(f) }

// VerifyPut reads back what a setter wrote, since shcl can take a value and
// read back something else. See "What the wrapper evens out" in design.md.
func VerifyPut(d Doc, path string, v Value) error {
	var st Status
	same := false
	switch v.Kind {
	case KindInt:
		g := d.GetInt(path)
		st, same = g.Status, g.Value == v.Int
	case KindFloat:
		g := d.GetFloat(path)
		st, same = g.Status, g.Value == v.Float
	case KindBool:
		g := d.GetBool(path)
		st, same = g.Status, g.Value == v.Bool
	case KindStr:
		g := d.GetStr(path)
		st, same = g.Status, g.Value == v.Str
	case KindStrs:
		g := d.GetStrs(path)
		st, same = g.Status, slices.Equal(g.Value, v.Strs)
		if st == Empty && len(v.Strs) == 0 {
			return nil
		}
	case KindInts:
		g := d.GetInts(path)
		st, same = g.Status, slices.Equal(g.Value, v.Ints)
		if st == Empty && len(v.Ints) == 0 {
			return nil
		}
	case KindFloats:
		g := d.GetFloats(path)
		st, same = g.Status, slices.Equal(g.Value, v.Floats)
		if st == Empty && len(v.Floats) == 0 {
			return nil
		}
	}
	if st == Good && same {
		return nil
	}
	msg := "reads back as " + st.String()
	if st == Good {
		msg = "reads back as a different value"
	}
	return &Error{Kind: ErrNoReadBack, Path: path, Msg: msg}
}

// VerifySyntax reads back what PutSyntax set at path. alone is a new
// document with the same text set at "k", and line is "k: text" parsed.
// shcl drops a trailing comment from the text, and 1.2 and 2.0 take some
// text a file line reads another way, so the two have to match. See "What
// the wrapper evens out" in design.md.
func VerifySyntax(d Doc, path string, alone, line Doc) error {
	if alone.Canonical() != line.Canonical() {
		return &Error{Kind: ErrNoReadBack, Path: path, Msg: "the text doesn't read back as typed"}
	}
	got, want := d.GetStr(path), line.GetStr("k")
	if got.Status != want.Status || got.Value != want.Value {
		return &Error{Kind: ErrNoReadBack, Path: path, Msg: "reads back as a different value"}
	}
	return nil
}

// DedupeNames keeps the first of each name, compared without ASCII case,
// since shcl matches names that way and lists one per line it was given on.
func DedupeNames(names []string) []string {
	out := make([]string, 0, len(names))
	seen := make(map[string]bool, len(names))
	for _, n := range names {
		k := strings.ToLower(n)
		if seen[k] {
			continue
		}
		seen[k] = true
		out = append(out, n)
	}
	return out
}

const formatHead = "##    Format   "

// ScanFormat is the number a `##    Format   N` line names, or 0. It's for
// adapters whose shcl predates the Format line, so they can refuse a newer
// file rather than misread it.
func ScanFormat(text string) int {
	for _, line := range strings.Split(text, "\n") {
		rest, ok := strings.CutPrefix(strings.TrimRight(line, "\r"), formatHead)
		if !ok {
			continue
		}
		n, err := strconv.Atoi(strings.TrimSpace(rest))
		if err == nil && n > 0 {
			return n
		}
	}
	return 0
}

// ReadText reads a whole file, with a missing file as ErrNotFound. A file
// that isn't valid UTF-8 is ErrIO, since no shcl version reads one right.
func ReadText(path string) (string, error) {
	b, err := os.ReadFile(path)
	if err == nil {
		if !utf8.Valid(b) {
			return "", &Error{Kind: ErrIO, Path: path, Msg: "not valid UTF-8"}
		}
		return string(b), nil
	}
	if errors.Is(err, fs.ErrNotExist) {
		return "", &Error{Kind: ErrNotFound, Path: path, Err: err}
	}
	return "", &Error{Kind: ErrIO, Path: path, Err: err}
}

// WriteAtomic writes data beside path and renames it into place, so a
// reader sees the old file or the new one, never half of one. An existing
// file's mode is kept.
func WriteAtomic(path, data string) error {
	mode := fs.FileMode(0o644)
	if st, err := os.Stat(path); err == nil {
		if !st.Mode().IsRegular() {
			return &Error{Kind: ErrIO, Path: path, Msg: "not a regular file"}
		}
		mode = st.Mode().Perm()
	}
	tmp, err := os.CreateTemp(filepath.Dir(path), "."+filepath.Base(path)+".tmp*")
	if err != nil {
		return &Error{Kind: ErrIO, Path: path, Err: err}
	}
	name := tmp.Name()
	fail := func(err error) error {
		tmp.Close()
		os.Remove(name)
		return &Error{Kind: ErrIO, Path: path, Err: err}
	}
	if _, err := tmp.WriteString(data); err != nil {
		return fail(err)
	}
	if err := tmp.Chmod(mode); err != nil {
		return fail(err)
	}
	if err := tmp.Sync(); err != nil {
		return fail(err)
	}
	if err := tmp.Close(); err != nil {
		os.Remove(name)
		return &Error{Kind: ErrIO, Path: path, Err: err}
	}
	if err := os.Rename(name, path); err != nil {
		os.Remove(name)
		return &Error{Kind: ErrIO, Path: path, Err: err}
	}
	return nil
}

// FormatFloat writes a float the same way in every wrapper language: the
// shortest digits that read back exactly, positional between 1e-5 and
// 1e17, else with an exponent.
func FormatFloat(f float64) string {
	switch {
	case math.IsNaN(f):
		return "nan"
	case math.IsInf(f, 1):
		return "inf"
	case math.IsInf(f, -1):
		return "-inf"
	}
	s := strconv.FormatFloat(f, 'e', -1, 64)
	mant, expText, _ := strings.Cut(s, "e")
	exp, _ := strconv.Atoi(expText)
	neg := strings.HasPrefix(mant, "-")
	digits := strings.Replace(strings.TrimPrefix(mant, "-"), ".", "", 1)
	return assembleFloat(neg, digits, exp)
}

func assembleFloat(neg bool, digits string, exp int) string {
	var b strings.Builder
	if neg {
		b.WriteByte('-')
	}
	switch {
	case exp >= 17 || exp < -5:
		b.WriteString(digits[:1])
		if len(digits) > 1 {
			b.WriteByte('.')
			b.WriteString(digits[1:])
		}
		b.WriteString("e")
		b.WriteString(strconv.Itoa(exp))
	case exp < 0:
		b.WriteString("0.")
		b.WriteString(strings.Repeat("0", -exp-1))
		b.WriteString(digits)
	case len(digits) <= exp+1:
		b.WriteString(digits)
		b.WriteString(strings.Repeat("0", exp+1-len(digits)))
		b.WriteString(".0")
	default:
		b.WriteString(digits[:exp+1])
		b.WriteByte('.')
		b.WriteString(digits[exp+1:])
	}
	return b.String()
}
