<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
<!-- TOC ignore:true -->
# Design

Status: draft, 2026-10-15. Go and Rust are built over shcl 1.2, 2.0, and 3.0. C and Python, and shcl 4, are in the backlog.

<!-- TOC ignore:true -->
## Table of contents

<!-- TOC -->

- [Summary](#summary)
- [Background](#background)
- [Goals](#goals)
	- [Non-goals](#non-goals)
- [Is it worth it](#is-it-worth-it)
- [Layout](#layout)
- [API](#api)
	- [Loading and saving](#loading-and-saving)
	- [Reads](#reads)
	- [Walking a document](#walking-a-document)
	- [Writes](#writes)
	- [Format and upgrade](#format-and-upgrade)
	- [Errors](#errors)
	- [Everything else](#everything-else)
- [What the wrapper evens out](#what-the-wrapper-evens-out)
- [Ops scripts](#ops-scripts)
- [Testing](#testing)
	- [Shared cases](#shared-cases)
	- [Fuzzing](#fuzzing)
- [Sharing the code](#sharing-the-code)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Copyright and license](#copyright-and-license)

<!-- /TOC -->

## Summary

- One small API over every shcl version a program might build against, the same in each language.

- A program picks its shcl version once, by import path in Go or cargo feature in Rust. Moving to another version is that one change, plus whatever its files need from `Upgrade`.

- The quirks each version has, and that each program used to work around on its own, are handled in one place.

- The rest of shcl is still there, through `Inner()`.

## Background

- shcl went from 1.2 to 2.0 to 3.0 in a few months, and 4 is planned soon after 3.0. 3.0 changes the file format a bit: arrays went from `a, b` to `[a, b]`, and selectors from `[#0]` to `(0)`. 4 keeps the format and changes the API a lot.

- Programs using shcl are spread across Go, Rust, C, C++, and Python, and over 1.2, 2.0, and several unreleased 3.0 commits. Most of them have their own adapter code for the same few problems.

## Goals

- The basics every config user needs, with the same names and the same behavior in every language and on every shcl version.

- Small enough that a fix is easy to make in every language at once. Each language is one module.

- Every behavior pinned by a test that runs the same script in every language and every shcl version.

- A way to reach the full shcl of a given version.

### Non-goals

- Covering all of shcl. Raw blocks, datetimes, durations, sizes, merging, schema generation, and the info block stay with shcl, through `Inner()`.

- Hiding the file format change. A program built on 1.2 can't read a format 3 file. The wrapper says so plainly, with `ErrTooNew`, and `Upgrade` moves files forward where the bound shcl can.

- Publishing to a package registry.

## Is it worth it

- Yes, at this size. shcl's 4 bindings are about 2.3 MB of source. The wrapper is about 1,000 lines per language, plus a test driver.

- Most of the cost is in testing: every language times every shcl version, with each version's pinned copy fetched and built.

- The wrapper can't help with what it doesn't cover. A program that leans on line-keeping saves, merges, or the info block still has those calls to move by hand when shcl changes them.

- It would stop being worth it if it grew toward shcl's own size. The line is the basics plus whatever two or more programs need.

## Layout

| Language | Where                                         | Pick the shcl version by
| :---     | :---                                          | :---
| Go       | `source/go/shclw`, plus `shclw1` to `shclw3` | import path: `.../source/go/shclw3`
| Rust     | `source/rust`, crate `shclw`                  | feature `shcl1` to `shcl3`, then module `shclw::v1` to `shclw::v3`

- Go can't import one package or another by condition inside one file, so each shcl version gets its own small package. They have the same calls, checked by the compiler against the `shclw.Doc` interface. The shared types and the ops runner are in `shclw`.

- Rust has one crate. Each shcl version is a module behind a feature, and the `ShclDoc` trait makes the compiler check that every module has every call. Several features can be on at once, for a tool that reads one version and writes another.

- Each version's code is a copy of the one before with its differences made, so a reader sees the whole adapter in one file.

## API

Go names are listed. Rust uses the same words in snake case, and `ShclDoc` has them as methods.

### Loading and saving

| Call                       | Does
| :---                       | :---
| `New()`                    | An empty document.
| `Parse(text)`              | Never fails. Problems are in `Diags()`.
| `ParseWith(text, level)`   | `Loose`, `Standard`, or `Strict`. Only `Strict` fails, with `ErrStrict`.
| `Open(path)`               | Reads and parses a file. A missing file is `ErrNotFound`, and a newer format is `ErrTooNew`.
| `OpenWith(path, level)`    | `Open` at a strictness.
| `d.Text()`                 | The document as it would be saved.
| `d.Canonical()`            | The document in canonical form.
| `d.Write(path)`            | Saves `Text()` atomically.
| `d.Diags()`, `d.Errors()`  | The load's diagnostics, and how many are errors.
| `d.Check(schema)`          | Validates against a schema document.
| `schema.DropDeclared(ds)`  | Drops the repeat hints for keys the schema says repeat.

### Reads

- 2 tiers, as in shcl's own design.
	- `d.Int(path, fallback)` gives the value, or the fallback when there's no good one.
	- `d.GetInt(path)` gives a `Got` with `Value`, `Status`, `Line`, and `Text`, the value as typed.

- Types: `Int`, `Float`, `Bool`, `Str`, `Strs`, `Ints`, `Floats`, and `Raw`, the value as typed with no coercion.

- Statuses: `Good`, `Empty`, `NotFound`, `BadType`, `Multiple`, and `BadPath`. 1.2 and 2.0 have no `BadPath` and say `NotFound`.

### Walking a document

| Call                  | Does
| :---                  | :---
| `d.Has(path)`         | Whether the path names anything.
| `d.Count(path)`       | How many instances it names.
| `d.Names(path)`       | Child names under it, once each, in file order.
| `d.Each(path)`        | Every instance, as a `Node` with `Path`, `Value`, and `Line`.
| `d.Child(base, name)` | `base` with one more name on the end, quoted if it needs it.
| `d.Line(path)`        | The line a single node is on.

- A `Node.Path` reads that instance and no other, whatever selector syntax the bound shcl uses. A program builds on it with `Child` and never writes a selector itself.

	~~~go
	for _, t := range d.Each("table") {
		name := d.Str(d.Child(t.Path, "name"), "")
	}
	~~~

### Writes

| Call                       | Does
| :---                       | :---
| `d.Put(path, v)`           | Sets an int, float, bool, string, or a slice of strings, ints, or floats.
| `d.PutSyntax(path, text)`  | Sets a value written in shcl's own syntax.
| `d.Comment(path, text)`    | Sets the comment above a path.
| `d.Remove(path)`           | Takes out a path and everything under it, and says how many instances went.

### Format and upgrade

- `FormatOf(text)` is the format a file's `##    Format   N` line names, or 0.

- `Upgrade(text, from)` and `UpgradeFile(path, from)` move a file to the bound shcl's format.
	- With `from` 0 the Format line decides, and a file with none is taken as current.
	- `from` 2 says the file is known to be from shcl 2.x or older. The file can't say so itself, so only the program can.
	- `UpgradeFile` copies the original beside it first, and `Backup` names the copy.
	- 1.2 and 2.0 have nothing to upgrade. They hand the text back, or refuse a newer file with `ErrTooNew`.

### Errors

- One error type, `*shclw.Error` in Go and `shclw::Error` in Rust, with a kind a program can act on: `NotFound`, `IO`, `Strict`, `Refused`, `BadPath`, `BadValue`, `TooNew`, `Unsupported`, `Ambiguous`, `BackupTaken`, and `NoReadBack`.

- `Refused` is a save that would drop lines the load couldn't use. 2.0 and 3.0 refuse those.

### Everything else

- `d.Inner()` is the shcl document underneath. Code that uses it is tied to that shcl version, which is the price of reaching past the wrapper.

- `ShclVersion` and `FormatMajor` say what's underneath.

- `shclw.Run` takes an ops script, described below. It's the test driver's engine, and also a way to run a list of edits written as text.

## What the wrapper evens out

Most of these were found by the tests or the fuzzer. Each has a test, either a shared case or an API test.

| Behavior                                   | shcl did                                                   | The wrapper does
| :---                                       | :---                                                       | :---
| `Names` on a path with several instances   | 1.2 and 2.0 give nothing. 3.0 lists every instance's children. | Lists every instance's children, everywhere.
| `Names` with a name given twice            | Lists it once per line.                                    | Lists it once.
| `Put` on a path with several instances     | 1.2 and 2.0 set something and say ok. 3.0 refuses.        | Refuses with `BadPath`, everywhere, and changes nothing.
| A put that doesn't read back the same      | Says ok.                                                   | Reads every put back. A mismatch is `NoReadBack`, with the edit left in place.
| Opening a newer format's file              | 1.2 and 2.0 read it, wrongly.                              | `ErrTooNew`.
| A file that isn't UTF-8                    | Varies by language.                                        | `ErrIO`, everywhere.
| Walking repeated entries                   | Selector syntax differs by version.                        | `Each` gives paths, and `Child` extends them.
| Saving in 1.2                              | No save call.                                              | `Write` saves atomically.
| Rust 1.2 fallback reads                    | No `get_*_or`.                                              | `int(path, fallback)` and the rest, as in every other version.

- An empty array reads back as `Empty` on 1.2 and 2.0, and as an empty `Good` on 3.0. The older formats can't write one any other way, so the wrapper accepts both.

## Ops scripts

An ops script is one op per line, with fields split by tabs. `\t`, `\n`, `\r`, and `\\` in a field are a tab, newline, return, and backslash. Blank lines and lines starting with `#` are skipped.

| Op                              | Prints
| :---                            | :---
| `parse [loose\|standard\|strict]` | `ok` or `error strict`, and the document is reloaded
| `text`, `canonical`             | the document, one `\|`-prefixed line per line
| `diags`, `errors`               | each diagnostic as line, severity, code, and message, or the count
| `get TYPE PATH`                 | status, line, and the value. Arrays print a count, then the items.
| `or TYPE PATH FALLBACK...`      | the value or the fallback
| `has`, `count`, `line`, `names` | the answer for PATH
| `each PATH`                     | index, line, and value, one instance per line
| `eachget TYPE PATH CHILD`       | `get` of CHILD under each instance
| `put TYPE PATH VALUE...`        | `ok` or `error KIND`
| `syntax PATH TEXT`, `comment PATH TEXT` | `ok` or `error KIND`
| `remove PATH`                   | how many instances went
| `check FILE [all\|drop]`         | the schema check, with the load's own diagnostics for `all`, and then dropped ones for `drop`
| `format`, `upgrade [FROM]`      | the format number, or the upgrade's result and text
| `write FILE`                    | `ok` and the saved file, or `error KIND`

- TYPE is `int`, `float`, `bool`, `str`, `strs`, `ints`, `floats`, or `raw`.

- Every op is echoed first, after a `>` and a space, so an output file reads as a transcript.

- Floats print the same in every language: the shortest digits that read back exactly, positional from 1e-5 to 1e17, otherwise with an exponent.

- `check` and `write` only work in the folder the driver is given, and refuse a name with a path in it.

## Testing

### Shared cases

- Each folder in `tests/cases` is a document, an ops script, and one `want.vN.txt` per shcl version. Go and Rust both run every case on every version and compare with the same want files, so the two languages can't drift apart.

- A file named `x.vK.ext` stands in for `x.ext` from version K up. So `doc.v1.shcl` is the document for 1.2 and 2.0, and `doc.v3.shcl` for 3.0, where the syntax differs.

- `SHCLW_UPDATE=1 go test ./shclw/` writes the want files. A changed want file is reviewed in the diff like code.

- Every feature and every fix gets a case, or a line in an existing one.

### Fuzzing

- `cicd/fuzz.py` makes a document and an ops script from each seed, with a generator that knows each format's syntax. A quarter of the documents are mutated copies of the case documents instead.

- Each case runs through the Go and Rust drivers on the same shcl version. The outputs must match byte for byte, neither may crash, and a put that says ok must read back the same.

- It runs for a wall-clock budget, and a failing seed is saved with its document, script, and both outputs. If too few reads find a value, the run fails, since a generator that stopped making readable documents would pass forever.

- Go's own fuzzer runs `Run` on random documents and scripts against every version, looking for panics.

- `cicd/cicd.bash` runs format, lint, tests, and both fuzzers. `--gate` runs all but the fuzzers, and is the pre-push hook.

## Sharing the code

- Go: `go get github.com/jim-collier/shcl-wrapper/source/go@<commit>`, then import the one version package. Copying `shclw` and the version package into a project works too.

- Rust: a git dependency on this repo with one feature on, or a copy of `source/rust/src`.

- A fix made in a copy goes back here, with a case, so every program gets it.

## Decisions

- The wrapper has its own small set of names, taking ideas from shcl 4's design rather than its exact names. Matching 4's names would have tied the wrapper to a draft that may still change.

- All 4 shcl versions are covered: 1.2, 2.0, 3.0, and 4 once it exists. 2.0 has no program running on it today, but some tests still write 2.0 files and also read them.

- Go uses one package per shcl version over build tags. With tags, forgetting the tag builds against a different shcl with no error. An import path can't be forgotten.

- Rust uses one crate with a module per version over separate crates, so a program that needs two versions at once can have both.

- Each version's adapter is a full copy with its differences, over one generic adapter with hooks. Most of each adapter is the differences.

- A put is always read back. shcl 4 plans the same check inside its setters, and until then the wrapper is the one place to do it for every version.

- MIT, since programs copy this code into themselves.

## Open questions

- When shcl 3.0's first beta is cut, the Go module path is still `.../v2`. Go won't take a `v3.0.0-beta1` tag on a `/v2` path, so the pin stays a commit until shcl's module path changes.

- shcl 4's API is a draft. Its adapter waits for 4 to exist in each language, which is planned Rust first, then C, then Go and Python.

## Copyright and license

> Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]<br />
> Licensed under the [MIT License](https://mit-license.org/)<br />
> SPDX-License-Identifier: `MIT`
