// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// Package shclw1 is the shclw wrapper over shcl 1.2.
//
// shcl 1.2 has no save, no keep-lines text, and no upgrade. Text gives the
// canonical form, Write saves it atomically, and Upgrade only refuses files
// that are newer than 1.2 can read.
package shclw1

import (
	"errors"
	"fmt"

	shcl "github.com/jim-collier/shcl/source/go"
	"github.com/jim-collier/shclwrap/source/go/shclw"
)

// ShclVersion is the shcl release this package wraps.
const ShclVersion = "1.2.0"

// FormatMajor is the newest file format this shcl reads.
const FormatMajor = 1

// Names from shclw, so a program needs only this import.
type (
	Status     = shclw.Status
	Strictness = shclw.Strictness
	Diag       = shclw.Diag
	Node       = shclw.Node
	Error      = shclw.Error
	ErrKind    = shclw.ErrKind
	Upgraded   = shclw.Upgraded
)

const (
	Good     = shclw.Good
	Empty    = shclw.Empty
	NotFound = shclw.NotFound
	BadType  = shclw.BadType
	Multiple = shclw.Multiple
	BadPath  = shclw.BadPath

	Loose    = shclw.Loose
	Standard = shclw.Standard
	Strict   = shclw.Strict
)

// KindOf is the ErrKind of err, or 0 when err didn't come from shclw.
func KindOf(err error) ErrKind { return shclw.KindOf(err) }

// Doc is a parsed document.
type Doc struct {
	d *shcl.Document
}

var _ shclw.Doc = (*Doc)(nil)

// Lib is this package's calls, for code that picks a version at run time.
var Lib = shclw.Lib{
	Name:        "shcl " + ShclVersion,
	FormatMajor: FormatMajor,
	New:         func() shclw.Doc { return New() },
	Parse:       func(text string) shclw.Doc { return Parse(text) },
	ParseWith:   func(text string, s Strictness) (shclw.Doc, error) { return ParseWith(text, s) },
	FormatOf:    FormatOf,
	Upgrade:     Upgrade,
}

// New is an empty document.
func New() *Doc { return &Doc{d: shcl.New()} }

// Parse never fails. Problems are in Diags.
func Parse(text string) *Doc { return &Doc{d: shcl.Parse(text)} }

// ParseWith fails only at Strict, with ErrStrict. The document comes back
// either way.
func ParseWith(text string, s Strictness) (*Doc, error) {
	d, err := shcl.ParseWith(text, strictness(s))
	if d == nil {
		d = shcl.New()
	}
	if err == nil {
		return &Doc{d: d}, nil
	}
	e := &shclw.Error{Kind: shclw.ErrStrict, Err: err}
	var le *shcl.LoadError
	if errors.As(err, &le) {
		e.Diags = diags(le.Diagnostics)
	}
	return &Doc{d: d}, e
}

// Open reads and parses a file. A missing file is ErrNotFound, and a file
// written for a newer format is ErrTooNew.
func Open(path string) (*Doc, error) { return OpenWith(path, shclw.Standard) }

// OpenWith is Open at a given strictness.
func OpenWith(path string, s Strictness) (*Doc, error) {
	text, err := shclw.ReadText(path)
	if err != nil {
		return New(), err
	}
	if f := FormatOf(text); f > FormatMajor {
		return New(), tooNew(path, f)
	}
	d, err := ParseWith(text, s)
	if e, ok := err.(*shclw.Error); ok {
		e.Path = path
	}
	return d, err
}

// FormatOf is the format a file's Format line names, or 0 with none.
func FormatOf(text string) int { return shclw.ScanFormat(text) }

// Upgrade has nothing to convert in 1.2. It hands text back, or refuses a
// file newer than 1.2 reads.
func Upgrade(text string, from int) (Upgraded, error) {
	f := max(FormatOf(text), from)
	if f > FormatMajor {
		return Upgraded{Text: text, From: f}, tooNew("", f)
	}
	return Upgraded{Text: text, From: FormatMajor}, nil
}

// UpgradeFile is Upgrade on a file. It never writes.
func UpgradeFile(path string, from int) (Upgraded, error) {
	text, err := shclw.ReadText(path)
	if err != nil {
		return Upgraded{}, err
	}
	u, err := Upgrade(text, from)
	if e, ok := err.(*shclw.Error); ok {
		e.Path = path
	}
	return u, err
}

// Inner is the shcl document underneath, for calls shclw doesn't cover.
func (d *Doc) Inner() *shcl.Document { return d.d }

// Text is the document as it would be saved.
func (d *Doc) Text() string { return d.d.ToCanonical() }

// Canonical is the document in canonical form.
func (d *Doc) Canonical() string { return d.d.ToCanonical() }

// Write saves the document atomically.
func (d *Doc) Write(path string) error { return shclw.WriteAtomic(path, d.d.ToCanonical()) }

// Diags is what the load found.
func (d *Doc) Diags() []Diag { return diags(d.d.Diagnostics()) }

// Errors counts error diagnostics.
func (d *Doc) Errors() int { return d.d.ErrorCount() }

// Check validates the document against a schema document.
func (d *Doc) Check(schema shclw.Doc) []Diag {
	s, ok := schema.(*Doc)
	if !ok {
		return []Diag{otherVersion}
	}
	return diags(d.d.Validate(s.d))
}

// DropDeclared takes out the repeat hints for keys this schema says repeat.
func (d *Doc) DropDeclared(ds []Diag) []Diag {
	return diags(shcl.SuppressDeclaredRepeats(d.d, undiags(ds)))
}

func (d *Doc) Int(path string, fallback int64) int64   { return d.d.GetIntOr(path, fallback) }
func (d *Doc) Bool(path string, fallback bool) bool    { return d.d.GetBoolOr(path, fallback) }
func (d *Doc) Str(path string, fallback string) string { return d.d.GetStringOr(path, fallback) }
func (d *Doc) Strs(path string, fallback []string) []string {
	return d.d.GetStringArrayOr(path, fallback)
}
func (d *Doc) Ints(path string, fallback []int64) []int64 { return d.d.GetIntArrayOr(path, fallback) }
func (d *Doc) Raw(path string, fallback string) string    { return d.d.GetRawOr(path, fallback) }

// Float and Floats go through GetFloat, so infinity gets the fallback too.
func (d *Doc) Float(path string, fallback float64) float64 {
	if g := d.GetFloat(path); g.Ok() {
		return g.Value
	}
	return fallback
}

func (d *Doc) Floats(path string, fallback []float64) []float64 {
	if g := d.GetFloats(path); g.Ok() {
		return g.Value
	}
	return fallback
}

func (d *Doc) GetInt(path string) shclw.Got[int64] { return got(d.d.ReadInt(path)) }
func (d *Doc) GetFloat(path string) shclw.Got[float64] {
	return shclw.FiniteFloat(got(d.d.ReadFloat(path)))
}
func (d *Doc) GetBool(path string) shclw.Got[bool]     { return got(d.d.ReadBool(path)) }
func (d *Doc) GetStr(path string) shclw.Got[string]    { return got(d.d.ReadString(path)) }
func (d *Doc) GetStrs(path string) shclw.Got[[]string] { return got(d.d.ReadStringArray(path)) }
func (d *Doc) GetInts(path string) shclw.Got[[]int64]  { return got(d.d.ReadIntArray(path)) }
func (d *Doc) GetFloats(path string) shclw.Got[[]float64] {
	return shclw.FiniteFloats(got(d.d.ReadFloatArray(path)))
}
func (d *Doc) GetRaw(path string) shclw.Got[string] { return got(d.d.ReadRaw(path)) }

func (d *Doc) Has(path string) bool  { return d.d.Exists(path) }
func (d *Doc) Count(path string) int { return d.d.Count(path) }
func (d *Doc) Line(path string) int  { return d.d.Line(path) }

// Names lists the child names under path once each, in file order. When
// path has several instances it's every instance's children, as in 3.0.
func (d *Doc) Names(path string) []string {
	if path == "" || d.d.Count(path) <= 1 {
		return shclw.DedupeNames(d.d.Children(path))
	}
	var all []string
	for _, n := range d.Each(path) {
		all = append(all, d.d.Children(n.Path)...)
	}
	return shclw.DedupeNames(all)
}

// Each is every instance path names, each with a path that reads it alone.
func (d *Doc) Each(path string) []Node {
	n := d.d.Count(path)
	vals := d.d.Instances(path)
	out := make([]Node, 0, n)
	for i := 0; i < n; i++ {
		p := path
		if n > 1 {
			p = fmt.Sprintf("%s[#%d]", path, i)
		}
		v := ""
		if i < len(vals) {
			v = vals[i]
		}
		out = append(out, Node{Path: p, Value: v, Line: d.d.Line(p)})
	}
	return out
}

// Child is base with one more name on the end, quoted if it needs it.
func (d *Doc) Child(base, name string) string {
	if base == "" {
		return shcl.QuoteSegment(name)
	}
	return base + "." + shcl.QuoteSegment(name)
}

// Put sets path to v. It takes ints, floats, bools, strings, and slices of
// strings, ints, and floats. A value that doesn't read back the same is
// ErrNoReadBack, with the edit left in place.
func (d *Doc) Put(path string, v any) error {
	val, err := shclw.Normalize(path, v)
	if err != nil {
		return err
	}
	if d.d.Count(path) > 1 {
		return &shclw.Error{Kind: shclw.ErrBadPath, Path: path, Msg: "Multiple"}
	}
	var ok bool
	switch val.Kind {
	case shclw.KindInt:
		ok = d.d.SetInt(path, val.Int)
	case shclw.KindFloat:
		ok = d.d.SetFloat(path, val.Float)
	case shclw.KindBool:
		ok = d.d.SetBool(path, val.Bool)
	case shclw.KindStr:
		ok = d.d.SetString(path, val.Str)
	case shclw.KindStrs:
		ok = d.d.SetStringArray(path, val.Strs)
	case shclw.KindInts:
		ok = d.d.SetIntArray(path, val.Ints)
	case shclw.KindFloats:
		ok = d.d.SetFloatArray(path, val.Floats)
	}
	if err := d.setErr(path, ok); err != nil {
		return err
	}
	return shclw.VerifyPut(d, path, val)
}

// PutSyntax sets path to text read as shcl value syntax. Text that doesn't
// read back as typed, such as one with a trailing comment, is
// ErrNoReadBack, with the edit left in place.
func (d *Doc) PutSyntax(path, text string) error {
	if d.d.Count(path) > 1 {
		return &shclw.Error{Kind: shclw.ErrBadPath, Path: path, Msg: "Multiple"}
	}
	if err := d.setErr(path, d.d.SetLiteral(path, text)); err != nil {
		return err
	}
	alone := New()
	// A failed set leaves alone empty, which the check reports.
	alone.d.SetLiteral("k", text)
	return shclw.VerifySyntax(d, path, alone, Parse("k: "+text+"\n"))
}

// Comment sets the comment above path.
func (d *Doc) Comment(path, text string) error { return d.setErr(path, d.d.SetComment(path, text)) }

// Remove takes out path and everything under it, and says how many
// instances went.
func (d *Doc) Remove(path string) int { return d.d.Remove(path) }

func (d *Doc) setErr(path string, ok bool) error {
	if ok {
		return nil
	}
	if r := d.d.WriteReason(path); r != shcl.Writable {
		return &shclw.Error{Kind: shclw.ErrBadPath, Path: path, Msg: r.String()}
	}
	return &shclw.Error{Kind: shclw.ErrBadValue, Path: path}
}

var otherVersion = Diag{Severity: shclw.SevError, Code: "W001", Message: "schema is from another shcl version"}

func tooNew(path string, f int) error {
	return &shclw.Error{Kind: shclw.ErrTooNew, Path: path,
		Msg: fmt.Sprintf("format %d, and shcl %s reads up to %d", f, ShclVersion, FormatMajor)}
}

func strictness(s Strictness) shcl.Strictness {
	switch s {
	case shclw.Loose:
		return shcl.Loose
	case shclw.Strict:
		return shcl.Strict
	}
	return shcl.Standard
}

func status(s shcl.Status) Status {
	switch s {
	case shcl.Good:
		return shclw.Good
	case shcl.Empty:
		return shclw.Empty
	case shcl.NotFound:
		return shclw.NotFound
	case shcl.BadType:
		return shclw.BadType
	case shcl.Multiple:
		return shclw.Multiple
	}
	return shclw.BadPath
}

func got[T any](r shcl.Read[T]) shclw.Got[T] {
	g := shclw.Got[T]{Value: r.Value, Status: status(r.Status), Line: r.Line}
	if r.Raw != nil {
		g.Text = *r.Raw
	}
	return g
}

func diags(in []shcl.Diagnostic) []Diag {
	out := make([]Diag, len(in))
	for i, d := range in {
		sev := shclw.SevHint
		if d.Severity == shcl.SeverityError {
			sev = shclw.SevError
		}
		out[i] = Diag{Line: d.Line, Severity: sev, Code: d.Code, Message: d.Message}
	}
	return out
}

func undiags(in []Diag) []shcl.Diagnostic {
	out := make([]shcl.Diagnostic, len(in))
	for i, d := range in {
		sev := shcl.SeverityHint
		if d.Severity == shclw.SevError {
			sev = shcl.SeverityError
		}
		out[i] = shcl.Diagnostic{Line: d.Line, Severity: sev, Code: d.Code, Message: d.Message}
	}
	return out
}
