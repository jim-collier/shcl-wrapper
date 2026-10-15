// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

package shclw_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/jim-collier/shclwrap/source/go/shclw"
	"github.com/jim-collier/shclwrap/source/go/shclw1"
	"github.com/jim-collier/shclwrap/source/go/shclw2"
	"github.com/jim-collier/shclwrap/source/go/shclw3"
)

func TestVerifyPut(t *testing.T) {
	for _, v := range libs {
		d := v.lib.Parse("a: 1\n")
		if err := shclw.VerifyPut(d, "a", shclw.Value{Kind: shclw.KindInt, Int: 1}); err != nil {
			t.Errorf("v%d: matching value: %v", v.n, err)
		}
		err := shclw.VerifyPut(d, "a", shclw.Value{Kind: shclw.KindInt, Int: 2})
		if shclw.KindOf(err) != shclw.ErrNoReadBack {
			t.Errorf("v%d: different value: got %v, want noreadback", v.n, err)
		}
		err = shclw.VerifyPut(d, "nope", shclw.Value{Kind: shclw.KindStrs})
		if shclw.KindOf(err) != shclw.ErrNoReadBack {
			t.Errorf("v%d: missing array: got %v, want noreadback", v.n, err)
		}
	}
}

func TestOpen(t *testing.T) {
	dir := t.TempDir()
	write := func(name, text string) string {
		p := filepath.Join(dir, name)
		if err := os.WriteFile(p, []byte(text), 0o644); err != nil {
			t.Fatal(err)
		}
		return p
	}
	newer := write("newer.shcl", "a: 1\n##    Format   9\n")
	binary := write("binary.shcl", "a: \xff\xfe\n")
	missing := filepath.Join(dir, "missing.shcl")
	opens := []struct {
		n    int
		open func(string) error
	}{
		{1, func(p string) error { _, err := shclw1.Open(p); return err }},
		{2, func(p string) error { _, err := shclw2.Open(p); return err }},
		{3, func(p string) error { _, err := shclw3.Open(p); return err }},
	}
	for _, o := range opens {
		for path, want := range map[string]shclw.ErrKind{newer: shclw.ErrTooNew, binary: shclw.ErrIO, missing: shclw.ErrNotFound} {
			if got := shclw.KindOf(o.open(path)); got != want {
				t.Errorf("v%d %s: got %v, want %v", o.n, filepath.Base(path), got, want)
			}
		}
	}
	if _, err := shclw1.Open(write("v3.shcl", "a: [x]\n##    Format   3\n")); shclw.KindOf(err) != shclw.ErrTooNew {
		t.Errorf("v1 on a format 3 file: got %v, want toonew", err)
	}
}
