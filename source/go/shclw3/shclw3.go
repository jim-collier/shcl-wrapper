// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// Package shclw3 is the shclw wrapper over shcl 3.0.
//
// Documents keep their lines as typed, so Text and Write give back what was
// read plus the edits, where 3.0 can manage that, and the canonical form
// where it can't. Rewrites says which. Write refuses with ErrRefused rather
// than drop a line.
package shclw3

import (
	"errors"
	"fmt"
	"os"

	"github.com/jim-collier/shclwrap/source/go/shclw"
	"github.com/jim-collier/shclwrap/source/go/shclw2"
	shcl "github.com/yottacore/shcl/source/go/v2"
)

// ShclVersion is the shcl release this package wraps.
const ShclVersion = "3.0.0-dev+ea0998fa"

// FormatMajor is the newest file format this shcl reads.
const FormatMajor = shcl.FormatMajor

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

// New is an empty document. It keeps lines like a loaded one, so it has
// none to lose and Rewrites says false.
func New() *Doc { return Parse("") }

// Parse never fails. Problems are in Diags.
func Parse(text string) *Doc {
	d, _ := ParseWith(text, shclw.Standard)
	return d
}

// ParseWith fails only at Strict, with ErrStrict. The document comes back
// either way.
func ParseWith(text string, s Strictness) (*Doc, error) {
	d, err := shcl.ParseKeepLines(text, strictness(s))
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
func FormatOf(text string) int {
	n, ok := shcl.FormatVersion(text)
	if !ok {
		return 0
	}
	return n
}

// Upgrade converts a file written for an older format. With from 0 the
// file's Format line decides, and a file with none is taken as current. Pass
// from 2 for a file known to be from shcl 2.x or older; it can't tell those
// apart by itself. A file that reads two ways is ErrAmbiguous, and one shcl
// 2.0 reports errors on is ErrRefused, with those errors in Diags.
func Upgrade(text string, from int) (Upgraded, error) {
	if f := max(FormatOf(text), from); f > FormatMajor {
		return Upgraded{Text: text, From: f}, tooNew("", f)
	}
	fromV2 := from > 0 && from < FormatMajor
	u := shcl.Upgrade(text, fromV2)
	out := upgraded(u)
	if u.Ambiguous > 0 {
		return out, &shclw.Error{Kind: shclw.ErrAmbiguous, Msg: fmt.Sprintf("%d lines read two ways", u.Ambiguous)}
	}
	if err := oldErrors(text, fromV2, u); err != nil {
		return Upgraded{Text: text, From: u.Format}, err
	}
	return out, nil
}

// UpgradeFile is Upgrade on a file. When it changes the file it first
// copies the original beside it, and Backup names the copy. A file that
// changed after it was read is left alone, and a failed write leaves the
// original in place (shclw.ReplaceRead).
func UpgradeFile(path string, from int) (Upgraded, error) {
	if from > FormatMajor {
		return Upgraded{From: from}, tooNew(path, from)
	}
	// A FIFO would block the read, and the write would put a file in place
	// of a device.
	if st, err := os.Stat(path); err == nil && !st.Mode().IsRegular() {
		return Upgraded{}, &shclw.Error{Kind: shclw.ErrIO, Path: path, Msg: "not a regular file"}
	}
	// One read for the 2.0 check, the conversion, and the backup. shcl's own
	// UpgradeFile would read the file a second time, after the check.
	text, err := shclw.ReadText(path)
	if err != nil {
		return Upgraded{}, err
	}
	u, err := Upgrade(text, from)
	if err != nil {
		if e, ok := err.(*shclw.Error); ok {
			e.Path = path
		}
		return Upgraded{From: u.From}, err
	}
	if !u.Changed {
		return u, nil
	}
	backup := func() (string, error) {
		name, err := shcl.WriteBackup(path, text, u.From)
		if err == nil {
			return name, nil
		}
		e := &shclw.Error{Kind: shclw.ErrIO, Path: path, Err: err}
		var ue *shcl.UpgradeError
		if errors.As(err, &ue) && ue.Kind == shcl.UpgradeBackupTaken {
			e.Kind = shclw.ErrBackupTaken
		}
		return "", e
	}
	write := func(path, data string) error {
		if err := shcl.WriteFileAtomic(path, data); err != nil {
			return &shclw.Error{Kind: shclw.ErrIO, Path: path, Err: err}
		}
		return nil
	}
	if u.Backup, err = shclw.ReplaceRead(path, text, u.Text, backup, write); err != nil {
		return Upgraded{From: u.From}, err
	}
	return u, nil
}

// oldErrors refuses a file taken as 2.x that shcl 2.0 reports errors on.
// The old program read those lines as it repaired them, or not at all, and
// an open raw block took the rest of the file. The upgrade would turn that
// into a clean file that means something else (design.md, Format and
// upgrade).
func oldErrors(text string, fromV2 bool, u shcl.Upgraded) *shclw.Error {
	if u.Ambiguous > 0 || (u.Current && !(fromV2 && u.Format < FormatMajor)) {
		return nil
	}
	var errs []Diag
	lines := map[int]bool{}
	for _, d := range shclw2.Parse(text).Diags() {
		if d.Severity == shclw.SevError {
			errs = append(errs, d)
			lines[d.Line] = true
		}
	}
	if len(errs) == 0 {
		return nil
	}
	return &shclw.Error{Kind: shclw.ErrRefused, Diags: errs, Msg: fmt.Sprintf("shcl 2.0 reports errors on %d lines; fix them and upgrade again", len(lines))}
}

func upgraded(u shcl.Upgraded) Upgraded {
	return Upgraded{Text: u.Text, From: u.Format, Changed: !u.Current, Unsure: u.Lost, Backup: u.Backup}
}

// Inner is the shcl document underneath, for calls shclw doesn't cover.
func (d *Doc) Inner() *shcl.Document { return d.d }

// Text is the document as it would be saved: its lines as typed, plus the
// edits, or the canonical form when 3.0 can't keep the lines.
func (d *Doc) Text() string {
	text, ok := d.d.ToTextKeepLines()
	if !ok {
		return d.d.ToCanonical()
	}
	return text
}

// Canonical is the document in canonical form.
func (d *Doc) Canonical() string { return d.d.ToCanonical() }

// Write saves Text atomically. It refuses with ErrRefused when the load
// dropped lines the save would lose.
func (d *Doc) Write(path string) error {
	_, err := d.d.SaveFileKeepLines(path)
	if err == nil {
		return nil
	}
	var refused *shcl.SaveRefused
	if errors.As(err, &refused) {
		return &shclw.Error{Kind: shclw.ErrRefused, Path: path,
			Msg: fmt.Sprintf("%d lines would be lost", refused.Lost), Err: err}
	}
	var we *shcl.WriteError
	if errors.As(err, &we) && we.Status == shcl.WriteNotFound {
		return &shclw.Error{Kind: shclw.ErrNotFound, Path: path, Err: err}
	}
	return &shclw.Error{Kind: shclw.ErrIO, Path: path, Err: err}
}

// Rewrites says whether Write puts the canonical form in place of the lines
// the document was loaded from. 3.0 does that when it can't keep them with
// the edits in, such as after a child is added under a flat dotted line,
// and comments can move. It builds the save's text to find out.
func (d *Doc) Rewrites() bool {
	_, kept := d.d.ToTextKeepLines()
	return !kept
}

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

// DropDeclared takes out the repeat and reopen hints for keys this schema
// says repeat.
func (d *Doc) DropDeclared(ds []Diag) []Diag {
	in := shcl.SuppressDeclaredRepeats(d.d, undiags(ds))
	return diags(shcl.SuppressDeclaredReopens(d.d, in))
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

// Names lists the child names under path once each, in file order.
func (d *Doc) Names(path string) []string { return shclw.DedupeNames(d.d.Children(path)) }

// Each is every instance path names, each with a path that reads it alone.
func (d *Doc) Each(path string) []Node {
	n := d.d.Count(path)
	vals := d.d.Instances(path)
	out := make([]Node, 0, n)
	for i := 0; i < n; i++ {
		p := path
		if n > 1 {
			p = fmt.Sprintf("%s(%d)", path, i)
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
	var st shcl.SetStatus
	switch val.Kind {
	case shclw.KindInt:
		st = d.d.SetInt(path, val.Int)
	case shclw.KindFloat:
		st = d.d.SetFloat(path, val.Float)
	case shclw.KindBool:
		st = d.d.SetBool(path, val.Bool)
	case shclw.KindStr:
		st = d.d.SetString(path, val.Str)
	case shclw.KindStrs:
		st = d.d.SetStringArray(path, val.Strs)
	case shclw.KindInts:
		st = d.d.SetIntArray(path, val.Ints)
	case shclw.KindFloats:
		st = d.d.SetFloatArray(path, val.Floats)
	}
	if err := d.setErr(path, st); err != nil {
		return err
	}
	return shclw.VerifyPut(d, path, val)
}

// PutSyntax sets path to text read as shcl value syntax. Text that doesn't
// read back as typed, such as one with a trailing comment, is
// ErrNoReadBack, with the edit left in place.
func (d *Doc) PutSyntax(path, text string) error {
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

func (d *Doc) setErr(path string, st shcl.SetStatus) error {
	switch st {
	case shcl.SetOk:
		return nil
	case shcl.SetBadPath, shcl.SetValueInPath, shcl.SetWildcard, shcl.SetNoSuchIndex, shcl.SetTooDeep,
		shcl.SetMultiple, shcl.SetUnderArray, shcl.SetHasChildren, shcl.SetRawBlock:
		return &shclw.Error{Kind: shclw.ErrBadPath, Path: path, Msg: st.String()}
	}
	return &shclw.Error{Kind: shclw.ErrBadValue, Path: path, Msg: st.String()}
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
