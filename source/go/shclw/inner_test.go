// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

package shclw_test

import (
	"slices"
	"strings"
	"testing"

	s3 "github.com/yottacore/shcl/source/go/v2"

	"github.com/jim-collier/shcl-wrapper/source/go/shclw"
	"github.com/jim-collier/shcl-wrapper/source/go/shclw1"
	"github.com/jim-collier/shcl-wrapper/source/go/shclw2"
	"github.com/jim-collier/shcl-wrapper/source/go/shclw3"
)

// innerSeen checks that an edit made through Inner, with a call the wrapper
// doesn't have, shows up in the wrapper's own reads and text.
func innerSeen(t *testing.T, n int, d shclw.Doc, paths []string) {
	t.Helper()
	if !slices.Contains(paths, "a") {
		t.Errorf("v%d: Inner().Paths() = %q, want it to list a", n, paths)
	}
	if g := d.GetStr("a"); g.Status != shclw.Empty {
		t.Errorf("v%d: after Inner().SetEmpty, a reads %v", n, g.Status)
	}
	if got := d.Str("b", ""); got != "set inside" {
		t.Errorf("v%d: after Inner().SetString, b reads %q", n, got)
	}
	if !strings.Contains(d.Text(), "set inside") {
		t.Errorf("v%d: Text() misses the inner edit:\n%s", n, d.Text())
	}
}

func TestInner(t *testing.T) {
	d1 := shclw1.Parse("a: 1\n")
	if !d1.Inner().SetEmpty("a") || !d1.Inner().SetString("b", "set inside") {
		t.Fatal("v1: an inner setter refused")
	}
	innerSeen(t, 1, d1, d1.Inner().Paths())

	d2 := shclw2.Parse("a: 1\n")
	if !d2.Inner().SetEmpty("a") || !d2.Inner().SetString("b", "set inside") {
		t.Fatal("v2: an inner setter refused")
	}
	innerSeen(t, 2, d2, d2.Inner().Paths())

	d3 := shclw3.Parse("a: 1\n")
	if d3.Inner().SetEmpty("a") != s3.SetOk || d3.Inner().SetString("b", "set inside") != s3.SetOk {
		t.Fatal("v3: an inner setter refused")
	}
	innerSeen(t, 3, d3, d3.Inner().Paths())
}
