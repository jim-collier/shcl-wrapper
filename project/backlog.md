<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->

<!-- TOC ignore:true -->
# Project backlog

<!-- TOC ignore:true -->
## Table of contents

<!-- TOC -->

- [Introduction](#introduction)
- [Issues](#issues)
- [Template](#template)
	- [New format](#new-format)

<!-- /TOC -->

## Introduction

The task list for the wrappers. Design and reasons are in [design.md](design.md).

Other projects are named here by stand-in names. The [project name map](../../private/project-names.md) has their repos.

## Issues

- Go and Rust wrappers over shcl 1.2, 2.0, and 3.0
	- ID: 2026101512024150
	- Type: Feature
	- Status: Waiting on signoff
	- Priority: High
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- Own small API, with ideas from shcl 4.
		- Every shcl version, with the version picked in one place.
		- A way to reach the rest of shcl.
		- A text command for anything the API doesn't bind.
	- Progress log:
		- 20261015: built, with `Inner()` for the rest of shcl and `Run` for ops scripts. The calls are listed in design.md.
		- 20261015: design.md lists where shcl versions behave differently and what the wrapper does instead.
	- Branch: go-rust
	- Test case: tests/cases 001 to 010, Go `api_test.go` and `inner_test.go`, Rust `tests/api.rs`

- CICD pipeline with shared cases and fuzzing
	- ID: 2026101512024152
	- Type: Feature
	- Status: Waiting on signoff
	- Priority: High
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- Fuzz and parity test every binding.
		- A regression test for every feature and fix.
	- Progress log:
		- 20261015: `cicd/cicd.bash` runs format, lint, tests, and fuzzing. `--gate` is the pre-push hook.
		- 20261015: Go and Rust read the same want files, so they can't drift.
		- 20261015: 15,000 fuzz cases in 90 seconds, Go against Rust, with no difference left.
	- Branch: go-rust

- PutSyntax doesn't read its value back
	- ID: 2026101513330007
	- Type: Bug
	- Status: Waiting on signoff
	- Severity: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Steps to reproduce:
		- PutSyntax `a # b` on a path.
	- Incorrect behavior: It says ok, and the value reads back as `a`.
	- Expected behavior: `NoReadBack`, the same as Put.
	- Note: Cobblevale reads its literal back and falls back to a plain string. Quillroute does the same for its single-quoted paths.
	- Reproduced: 20261015, on 1.2, 2.0, and 3.0, in Go and Rust. `x#y` loses `#y` the same way.
	- Actual cause:
		- shcl reads the text as a value plus a trailing comment and drops the comment. The wrapper never read PutSyntax back.
		- On 1.2 and 2.0, PutSyntax on a path with several instances also set the first one and said ok, where Put refuses.
	- Actual fix: PutSyntax now sets the text in a new document and parses `k: text` in another, and the two have to match. The value at the path has to read the same as the parsed one. Otherwise it's `NoReadBack`, with the edit left in place. On 1.2 and 2.0 a path with several instances is `BadPath`, as with Put.
	- Progress log:
		- 20261015: the check also catches `[a, b]` on 1.2 and 2.0, which sets `"[a", "b]"` while a file line `k: [a, b]` reads another way. design.md lists both.
	- Branch: floats
	- Commit: 9b835f5
	- Test case: tests/cases/012-syntax. It failed before the fix in Go and Rust on every version, and passes after.
	- Swept: PutSyntax in Go `shclw1` to `shclw3` and Rust `v1` to `v3`. Comment has the same several-instances gap on 1.2 and 2.0, logged as 2026101514071600.
	- Acceptance signoff: Waiting. The check is wider than the item asked, since it also flags 1.2 and 2.0 text a file reads another way.

- Float reads take infinity as good
	- ID: 2026101513330008
	- Type: Bug
	- Status: Waiting on signoff
	- Severity: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Steps to reproduce:
		- Read `x: 1e400` as a float.
	- Incorrect behavior: `Good`, with infinity as the value.
	- Expected behavior: `BadType`.
	- Note: Wovenshell checks every float it reads. Cobblevale found a `NaN` put on 1.2 that reads back as `BadType`.
	- Reproduced: 20261015, on 1.2 and 2.0, in Go and Rust. 3.0 already says `BadType`, and the fallback reads gave infinity too.
	- Actual cause:
		- shcl 1.2 and 2.0 read a number too big for a float as infinity. A put of infinity or NaN wrote it, then read it back as `BadType` and said `NoReadBack`, with the value left in the file.
		- The ops scripts read `1e400` as infinity in Rust and as a bad value in Go.
	- Decisions:
		- A put of infinity or NaN is refused with `BadValue` in every version and changes nothing, as 3.0 does. design.md has it under Decisions.
		- An array with a bad item reads as `BadType` and keeps its length, with 0 for the bad items, as 3.0 does.
	- Actual fix: Float reads that find infinity or NaN say `BadType`, and the fallback reads give the fallback. Puts refuse them up front. Ops scripts take `nan`, `inf`, and `-inf`, and read a number too big as infinity in both languages.
	- Progress log:
		- 20261015: the fuzzer now makes these values and checks that no `Good` float read gives one.
	- Branch: floats
	- Commit: 9b835f5
	- Test case: tests/cases/011-floats, plus the NaN and infinity half of Go `TestPutFloat32` and Rust `put_f32`. They failed before the fix on 1.2 and 2.0 in both languages, and pass after. On 3.0 only Go's 011 failed, on the ops parser, since 3.0 already refused these.
	- Swept: GetFloat, GetFloats, Float, and Floats in Go `shclw1` to `shclw3` and Rust `v1` to `v3`. Put for float, float32, and float slices in both. Ints have no such value.
	- Acceptance signoff: Waiting. Refusing a NaN or infinity put was a choice.

- Move the 3.0 pin to the first 3.0 beta
	- ID: 2026101512024165
	- Type: Task
	- Status: Stalled
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- Pin `v3.0.0-beta1` once it's tagged, in place of a dev commit.
	- Notes:
		- The tag is spelled with no dot, per shcl's own backlog.
		- shcl's Go module path is still `.../v2`, and Go won't take a v3 tag on it. Go stays on a commit until that changes.
		- Quillroute plans to have its release refuse a commit pin. The same check would fit here.

- Upgrade passes along a conversion that stopped partway
	- ID: 2026101513330001
	- Type: Bug
	- Status: Waiting on signoff
	- Severity: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Related IDs: 2026101513330002
	- Steps to reproduce:
		- Upgrade a 2.x file that ends inside a raw block that never closes.
	- Incorrect behavior: The text comes back unconverted and with no Format line. Nothing is counted as lost, and there's no error.
	- Expected behavior: An error, and the file left alone.
	- Note: Radixmill refuses any result that doesn't read as the current format. Wovenshell runs shcl's stamped conversion a second time to find out.
	- Reproduced: 20261015, on 3.0 in Go and Rust, with `from` 0 and 2. The text did come back converted. shcl closed the block at the end of the file and added the info block, so every line after the open fence stays raw text for good. Nothing lost and no error, as the item says. The unconverted text with no Format line is what shcl's own `Migrate` gives.
	- Actual cause:
		- shcl's `Upgrade` writes a raw block that never closes as closed, and says nothing. Its `migrate` command refuses the same file. The wrapper passed shcl's answer on.
	- Decisions:
		- Refused, as the item asks. The rule is wider than raw blocks: any file taken as 2.x that shcl 2.0 reports an error on is refused, which covers this and 2026101513330002. design.md has it under Decisions.
	- Actual fix: On 3.0, `Upgrade` and `UpgradeFile` parse a file they take as 2.x with shcl 2.0 first. Any error there is `Refused`, with 2.0's errors in `Diags`. `UpgradeFile` then writes nothing and makes no backup. The 3.0 adapter builds 2.0 in now, and the Rust `shcl3` feature turns on `shcl2`.
	- Note: shcl friction, a silent wrong answer. `a: 1`, then `b: ```` and `raw`, with no closing fence. `shcl upgrade --write` prints E005 and writes a fresh file with the block closed at the end, at exit 0. `migrate` refuses the same file at exit 7. Expected: `upgrade` and `Upgrade` refuse it too. Checked on shcl's current dev; the library hasn't changed since the pin.
	- Verified: the gate and a 20 second fuzz, Go against Rust, with no difference.
	- Branch: upgrade
	- Commit: 18041cf
	- Test case: tests/cases/013-upgrade-raw, Go `TestUpgradeRefusesOldErrors`, Rust `upgrade_refuses_old_errors`. They failed before the fix in Go and Rust, and pass after.
	- Swept: `Upgrade` and `UpgradeFile` in Go `shclw3` and Rust `v3`. The 1.2 and 2.0 adapters convert nothing. 17 files ending in an open block, with `from` 0 and 2, were all refused or `Ambiguous`, the same in Go and Rust.
	- Acceptance signoff: Waiting. The rule is wider than the item, and the 3.0 adapter now builds shcl 2.0 in.

- Upgrade passes along lines the old shcl refused
	- ID: 2026101513330002
	- Type: Bug
	- Status: Waiting on signoff
	- Severity: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Related IDs: 2026101513330001
	- Steps to reproduce:
		- Upgrade `x: [ab]` as a 2.x file.
	- Incorrect behavior: It comes back as `x: ab`, with nothing lost. 1.2 and 2.0 refused that line, so the old program never read it.
	- Expected behavior: The line counts as lost, or the upgrade refuses.
	- Note: Radixmill refuses a file whose original text has any E019 line. shcl means it to work this way.
	- Reproduced: 20261015, on 3.0 in Go and Rust, with `from` 2. One detail differs. 1.2 and 2.0 report the line as E015, "missing colon; repaired as an empty value", and still read `x` as `ab`, so the old program read it with an error, and a Strict load failed. E019 is a 3.0 code. A line 2.0 skipped outright, like `z: [ab] c`, came through as written with nothing lost too.
	- Actual cause:
		- shcl's conversion reads `x: [ab]` as the 2.x `name:[disc]` form, the same way 2.0 repaired it, and doesn't look at whether 2.x reported an error on the line. Lines 2.x skipped come through as written.
	- Decisions:
		- The upgrade refuses, rather than count the line in `Unsure`. A count is easy to skip, and the upgraded file loads clean, so nothing would point at the line again.
		- Any error shcl 2.0 reports counts, whatever its code, since E019 doesn't exist before 3.0. The same rule covers 2026101513330001. design.md has it under Decisions.
		- It applies to a file the upgrade takes as 2.x: one shcl converts, or one passed with `from` 2 that doesn't name format 3. A file that loads clean with `from` 0 is still left alone with no check.
	- Actual fix: The same change as 2026101513330001. On 3.0, a file taken as 2.x that shcl 2.0 reports errors on is `Refused`, with 2.0's errors in `Diags`, and `UpgradeFile` writes nothing.
	- Note: shcl friction, a silent wrong answer. `x: [a, b]` upgraded with fromV2 comes back current with nothing lost. 2.0 read it as one string, `a, b`, and 3.0 reads 2 items. shcl's conversion counts it lost, but `Upgrade` drops that count when the file loads clean and the conversion leaves its text alone. Expected: Lost 1. The wrapper now refuses this file anyway, since 2.0 reports E015 on it.
	- Verified: the gate and a 20 second fuzz, Go against Rust, with no difference.
	- Branch: upgrade
	- Commit: 18041cf
	- Test case: tests/cases/014-upgrade-errors, Go `TestUpgradeRefusesOldErrors`, Rust `upgrade_refuses_old_errors`. They failed before the fix in Go and Rust, and pass after.
	- Swept: `Upgrade` and `UpgradeFile` in Go `shclw3` and Rust `v3`. The 1.2 and 2.0 adapters convert nothing. A format 3 file passed with `from` 2 that 2.0 has errors on, such as `tags: [a, b]`, is still left alone.
	- Acceptance signoff: Waiting. Refusing was picked over counting, and the item's E019 premise didn't hold as written.

- Info block and Format line
	- ID: 2026101512024154
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- Write and remove shcl's info block, and stamp the Format line.
	- Notes:
		- Quillroute, Radixmill, Folderkite, Tickerwren, and Mossrealm each do this by hand today.
		- 1.2 and 2.0 have no info block, so the wrapper would write it there itself.
		- Cobblevale wants a way to stamp the Format line without the banner. Its new files had none, and read back like 2.x files.
		- shcl's own banner setter misses a block that has a line after it, and adds a second one. Folderkite strips the old block itself.
		- Banners written during the 3.0 betas link a tag that doesn't exist. Wovenshell replaces 4 old spellings at launch.
		- Radixmill and Cobblevale add the Format line to their test files by hand.

- C wrapper
	- ID: 2026101512024159
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- One header over `shcl.h`, usable from C++.
		- The shcl version picked at compile time.
		- Its own driver, run against the shared cases and in the fuzzer.
	- Notes:
		- Cobblevale uses it from C++, and Folderkite from C.
		- shcl's C header has no version macro before 3.0, so 1.2 and 2.0 are told apart by what they declare.

- shcl 4 adapters
	- ID: 2026101512024163
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 2026-10-15
	- Opened by: JC
	- Prereq IDs: shcl 4 in each language. It's planned Rust first, then C, then Go and Python.
	- Requirements:
		- The same API, over shcl 4.
	- Notes:
		- 4 keeps every line as typed, so Text and Write get simpler. Its read-back check in setters can replace the wrapper's.

- Say when Write rewrote the whole file
	- ID: 2026101513330003
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Write, or a call made before it, says when 3.0 couldn't keep the file's own lines and wrote the canonical form instead.
	- Note: Quillroute compares the canonical text with what it read, and adds an "also" line to its plan. Wovenshell checks renames against the canonical form, since the fallback moves comments.

- Pick the first or last of a repeated key
	- ID: 2026101513330004
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- A read option that takes the first or the last instance, in place of `Multiple`.
		- A way to list the other instances' lines, for a "set more than once" warning.
	- Note: Tallowbase takes the first. Quillroute takes the last and lists the rest as ignored. Folderkite falls back to the default with a warning. Wovenshell, Radixmill, and Tickerwren report the key as set twice.

- UpgradeFile puts the old file back when the write fails
	- ID: 2026101513330005
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- The original survives any failure.
		- Before replacing the file, check that it hasn't changed since it was read.
	- Note: Radixmill, Quillroute, Mossrealm, and Cobblevale each do this themselves. Check what shcl's own UpgradeFile already does first.

- C: copy reads out and free shcl's read memory
	- ID: 2026101513330046
	- Type: Feature
	- Status: Queued
	- Priority: High
	- Opened: 20261015-133300
	- Opened by: JC
	- Parent ID: 2026101512024159
	- Note: shcl keeps every string read until the document is freed. Folderkite saw 19 MB after 200,000 reads, so it releases reads after each one and compacts after a save.

- Layered loading
	- ID: 2026101512024156
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Related IDs: 2026101513330030
	- Requirements:
		- Defaults, then site, then user file, as one document.
	- Notes:
		- Cobblevale uses shcl's merge. Mossrealm does it by hand.
		- Mossrealm's layers are defaults, then a system file, then the user file, then the command line.

- Python wrapper
	- ID: 2026101512024161
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- One module. It finds which shcl is installed at run time.
		- Its own driver, run against the shared cases and in the fuzzer.
	- Notes:
		- Used by Lenscart, on 1.2.

- Move the other projects onto the wrapper
	- ID: 2026101512024167
	- Type: Task
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Prereq IDs: 2026101512024150, and the C or Python wrapper where a project needs one.
	- Requirements:
		- One project at a time, deleting its own adapter code as it goes.
	- Notes:
		- Tallowbase, Tickerwren, Radixmill, and Mossrealm fit the API as it is.
		- Quillroute and Wovenshell also need the info block, and Wovenshell datetime reads.
		- Cobblevale and Folderkite wait on the C wrapper, and Lenscart on the Python one.
		- The wrapper already covers some of their workarounds, which can go as each one moves: Tallowbase's path cursor and index walks, Quillroute's name dedupe, quoting names made of digits, refusing a set on a repeated key, reading every put back, and failing a load on any error.
		- Each project's other workarounds have their own items, with the project named in the item.

- Tests for upgrade and save failures
	- ID: 2026101512024169
	- Type: Task
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- A refused save, a taken backup name, and an upgrade of a real file, on each version that has them.

- Report blocks that shcl merged because they share a name
	- ID: 2026101513330009
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- List each merged block, with the lines of both copies.
	- Note: Tallowbase, Cobblevale, and Radixmill all lost a second copy of a block without a word. shcl's H002 hint only fires when the 2 blocks are apart.

- Tell a 2.x file from a current one by its banner
	- ID: 2026101513330010
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- With `from` 0, a file with no Format line but with the 2.x banner is taken as 2.x.
	- Note: Quillroute looks for the banner. Cobblevale, Folderkite, Mossrealm, and Wovenshell take any file with no Format line as old, which is wrong for a new file with no banner.

- Upgrade swaps the 2.x banner for the current one
	- ID: 2026101513330011
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Replace the old banner block in place, keeping the file's line endings.
	- Note: shcl leaves the old banner and adds a Format line at the end. Quillroute replaces the block itself.

- Upgrade lists the lines it changed
	- ID: 2026101513330012
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Quillroute diffs the text line by line to say how many lines were written differently.

- Upgrade lists the lines it lost
	- ID: 2026101513330013
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Line numbers, not only a count.
	- Note: Cobblevale logs them, and its CI checks the count. Wovenshell shows a notice for each one. A 1.x line break in a bare value that looks like a Windows path counts as lost, though 3.0 could write it as `\u000A`.

- Open a newer-format file read-only
	- ID: 2026101513330014
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Open can read a newer file where the bound shcl is able to, and Write refuses on that document.
	- Note: Open refuses with `ErrTooNew` today. Folderkite reads newer files and never saves over them. Radixmill never rewrites a file named on the command line. Cobblevale stops writing when its backup failed.

- Create-only Write
	- ID: 2026101513330015
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Fails if the file exists, and stays right when 2 processes create it at once.
	- Note: Radixmill hard-links a synced temp file into place. shcl's atomic write replaces a file it finds there.

- Write with a file mode
	- ID: 2026101513330016
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- A new file is created with the mode given, such as 0600.
	- Note: Quillroute's accounts file has tokens in it. shcl creates files at 0666 less the umask.

- Write refuses when the file changed on disk since it was read
	- ID: 2026101513330017
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Quillroute re-reads the file under its own lock. Radixmill and Lenscart both lost writes to 2 processes saving at once, and Lenscart moved its counters out of shcl because of it.

- List the lines a load couldn't use
	- ID: 2026101513330018
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Line numbers, not only a count.
	- Note: Wovenshell builds the list from error diagnostics and drops line 0. Folderkite warns at load with the count. Cobblevale only tracks whether there were any.

- Back up a file before a save that would lose lines
	- ID: 2026101513330019
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- A save option that copies the file aside and then writes, in place of `Refused`.
	- Note: Cobblevale does this on its first write over such a file. 1.2 has no refusal, so the wrapper's own 1.2 Write loses those lines with no warning.

- Put fills in a commented default line
	- ID: 2026101513330020
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Setting a key whose commented `key: value ## Default` line is in the file fills in that line, so the lines below it don't move.
	- Note: shcl adds a new line above the comment. Wovenshell writes the line by hand, and reprints its launch messages with the new line numbers. Folderkite rebuilds its whole defaults list on every save.

- Read the commented defaults in a file
	- ID: 2026101513330021
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- List commented `key: value` lines, each with the path it would set.
	- Note: shcl gives comment lines no path. Wovenshell has its own indent walker for this, and Folderkite strips and re-adds its list.

- Comment doesn't stack, and leaves the user's own notes
	- ID: 2026101513330022
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Comment replaces the text it wrote before, and leaves other comments above the key alone.
	- Note: Each set adds another line, and clearing would delete notes the user wrote. Folderkite only sets a comment on a new key, so a reworded comment never reaches a key that's already set.

- Remove keeps a key's comments when it's set again
	- ID: 2026101513330023
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Wovenshell edits the lines by hand so that way a setting goes back to its default. Mossrealm lives with losing the comment.

- Show the text a value would be written as
	- ID: 2026101513330024
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Quillroute and Wovenshell set the value on a throwaway document and read its canonical text. Folderkite builds its defaults list the same way. Quillroute also checks that text for `\u` escapes.

- Put writes Windows paths in single quotes
	- ID: 2026101513330025
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- A string with a backslash and no apostrophe or line break goes in single quotes.
	- Note: shcl writes some paths bare, some in single quotes, and some in double quotes with every backslash doubled. Quillroute uses PutSyntax and a read-back. It's queued with shcl too.

- Let a program treat chosen hints as errors
	- ID: 2026101513330026
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Cobblevale counts errors only, so a mistyped table that merges every entry shows nothing. In Quillroute, `p: "C:\temp"` loads with a tab in it, and only hint H004 says so.

- Check keeps schema faults apart from the file's
	- ID: 2026101513330027
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: A schema fault's line number is in the schema, not the file. On 1.2 it also turns off the unknown-key check. Tallowbase splits them out and tests that an unknown key still gets reported.

- Names as the file typed them
	- ID: 2026101513330028
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Names and Each can give a name in its original case.
	- Note: Radixmill's error messages can only show the lowercased name. Lenscart, Tallowbase, and Quillroute store names as values to keep their case.

- Lists of records
	- ID: 2026101513330029
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Requirements:
		- Write and read a list of rows with named fields. A write replaces the whole list.
	- Note: Lenscart removes the section and writes `rowN` blocks. Mossrealm packs records into one flat float array. Cobblevale uses repeated blocks and walks them by index.

- Save only what differs from the layers below
	- ID: 2026101513330030
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Related IDs: 2026101512024156
	- Note: Mossrealm writes only the differences, and deletes the user file when nothing differs. Folderkite stores only changed values.

- Int reads within a range
	- ID: 2026101513330031
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: shcl ints are 64-bit. Mossrealm guards u32 reads and Cobblevale int32 reads, and both fall back when a value is out of range.

- Unsigned 64-bit values
	- ID: 2026101513330032
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Mossrealm has seeds past the i64 limit, and stores them as base32 text.

- C: strings with lengths
	- ID: 2026101513330047
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Parent ID: 2026101512024159
	- Note: shcl strings aren't NUL-terminated and can hold a NUL. Cobblevale garbled non-ASCII text, and Folderkite cut a save short at a NUL.

- C: build with shcl's file calls left out
	- ID: 2026101513330048
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Parent ID: 2026101512024159
	- Note: Cobblevale compiles shcl in one file with `SHCL_NO_FILE_IO`, since the file calls pull in `windows.h`, adn writes files itself. Write would need its own code for that build.

- Comment on a path with several instances changes the first one on 1.2 and 2.0
	- ID: 2026101514071600
	- Type: Bug
	- Status: Queued
	- Severity: Low
	- Opened: 20261015-140716
	- Opened by: JC
	- Related IDs: 2026101513330007
	- Steps to reproduce:
		- Comment on `n` in a file with `n: 1` and `n: 2`, on 1.2 or 2.0.
	- Incorrect behavior: It says ok and puts the comment above the first `n`.
	- Expected behavior: `BadPath`, as 3.0 does and as Put and PutSyntax do.

- The Rust shared-case test leaves its scratch folders behind
	- ID: 2026101514214605
	- Type: Bug
	- Status: Queued
	- Severity: Low
	- Opened: 20261015-142146
	- Steps to reproduce:
		- Run `cargo test --all-features --test cases`.
	- Incorrect behavior: A `shclw-rust-cases-PID` folder with one folder per case and version stays in the temp folder after every run.
	- Expected behavior: It's removed at the end, as Go's `t.TempDir()` does.

- Commit and publish from cicd
	- ID: 2026101512024171
	- Type: Task
	- Status: Queued
	- Priority: Low
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- `-m` and the usual commit, backup, and publish stage, as the other projects have.
		- The pre-push hook runs the whole gate for a push that only deletes a branch. It should skip that.

- Run the tests on Windows and macOS
	- ID: 2026101512024174
	- Type: Task
	- Status: Queued
	- Priority: Low
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- The shared cases on Windows and macOS, mostly for the atomic write.
	- Notes:
		- shcl 2.0 on Windows could lose the old file when a replace failed. 3.0 fixed it, and Wovenshell still keeps its own restore. The wrapper's own 1.2 write needs the same check.

- Comment out a setting in place
	- ID: 2026101513330033
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Wovenshell turns a setting off by commenting its line by hand, since Remove takes the comments with it.

- Reorder repeated entries
	- ID: 2026101513330034
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Wovenshell removes and rewrites a whole block to change its order.

- Remove takes out a header it left empty
	- ID: 2026101513330035
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Removing the last key under a repeated header leaves the header, and takes the blank line above it. Quillroute lives with it. It's queued with shcl too.

- Put replaces every copy of a repeated key
	- ID: 2026101513330036
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: The wrapper refuses with `BadPath` today. Cobblevale removes the key and then sets it.

- Write skips a save when nothing changed
	- ID: 2026101513330037
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Wovenshell compares the text first.

- Read a number written as an int or a float
	- ID: 2026101513330038
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Wovenshell merges the 2 for single values and for mixed lists.

- Decimal reads
	- ID: 2026101513330039
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Tickerwren reads money as text and parses it itself, so no binary float gets in.

- Warn on a value that's only a comment
	- ID: 2026101513330040
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: `color: #112233` reads as `Empty`. Wovenshell finds those lines and names them.

- Warn on lines under a key that has a value
	- ID: 2026101513330041
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Those lines are never read, and the key above reads as `Empty`. Radixmill refuses them. In Folderkite a file written that way loads as nothing, with no complaint.

- Map a struct to a block
	- ID: 2026101513330042
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Mossrealm maps every field by hand, both ways. This may be past the size design.md sets for the wrapper.

- Check a copied wrapper against its commit
	- ID: 2026101513330043
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Copying is one of the supported ways to use the wrapper. Radixmill byte-checks its copy of shcl against the pinned commit, after a tree-wide edit broke it once.

- Find out if array reads take a single value on every version
	- ID: 2026101513330044
	- Type: Task
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Mossrealm writes a one-item row as a plain value and reads it both ways. Lenscart reads every field as an array.

- Find out if Check repeats the load's diagnostics
	- ID: 2026101513330045
	- Type: Task
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Note: Lenscart drops repeats by line, code, and message after validating.

- C: out-of-memory hook
	- ID: 2026101513330049
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Parent ID: 2026101512024159
	- Note: shcl 1.2 and 2.0 print and exit on out of memory. Folderkite sets its own hook.

- Python: run from a copied file
	- ID: 2026101513330050
	- Type: Feature
	- Status: Queued
	- Priority: Low
	- Opened: 20261015-133300
	- Opened by: JC
	- Parent ID: 2026101512024161
	- Note: Lenscart bundles shcl's wheel in its packages. Folderkite parses shcl by hand in its Python tools, since nothing is installed there.

- Rename the project shclwrap
	- ID: 2026101513524769
	- Type: Task
	- Status: Done
	- Priority: Critical
	- Opened: 20261015-135247
	- Opened by: JC
	- Requirements:
		- The project, the GitHub repo, and the Go module path are named shclwrap.
	- Decisions:
		- The Go packages, the Rust crate, the drivers, and the `SHCLW_` variables stay `shclw`.
		- The contact address moves to the new name.
		- The local project folders get renamed by hand, outside the repo.
	- Progress log:
		- 20261015: the name is shclwrap in the docs, the cicd output, and the contact address. The Go module path is `github.com/jim-collier/shclwrap/source/go`, with every import moved.
		- 20261015: the GitHub repo is now jim-collier/shclwrap. GitHub sends the old URL there.
	- Branch: shclwrap
	- Commit: 3f6eb38
	- Test case: none of its own, since it's a rename. The cicd gate passes on the renamed module.
	- Swept: `git grep -iE 'shcl[-_]wrapper'` finds nothing.
	- Closed: 20261015-135423
	- Acceptance signoff: Self-closed: mechanical rename.

- Put writes a float32 with float64 noise
	- ID: 2026101513330006
	- Type: Bug
	- Status: Done
	- Severity: Avg
	- Opened: 20261015-133300
	- Opened by: JC
	- Steps to reproduce:
		- Put 0.9 as a Go `float32` or a Rust `f32`.
	- Incorrect behavior: The file says `0.8999999761581421`.
	- Expected behavior: `0.9`, the shortest text that reads back as the same float32.
	- Note: Mossrealm formats the value as text and parses it back before setting it. Wovenshell rounds to 3 places.
	- Reproduced: 20261015, on 1.2, 2.0, and 3.0, in Go and Rust.
	- Actual cause:
		- The wrapper widened a float32 to float64 before setting it, and shcl wrote every digit of the wider value.
	- Actual fix: A float32 goes in as its own shortest digits, read as a float64.
	- Branch: floats
	- Commit: 9b835f5
	- Test case: Go `TestPutFloat32`, Rust `put_f32`. Both failed before the fix and pass after. The ops scripts have no float32 type.
	- Swept: Go `Normalize` is the only float32 path. Rust has only `From<f32>`. Neither takes a float32 slice.
	- Closed: 20261015-140716
	- Acceptance signoff: Self-closed: did what the item asked, and its test failed before and passes after.

- Make the repo public
	- ID: 2026101512024148
	- Type: Task
	- Status: Done
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- While the repo is private, `go get` needs `GOPRIVATE=github.com/jim-collier/*`, and a cargo git dependency needs `net.git-fetch-with-cli = true`.
		- Public would need neither.
	- Decisions:
		- Private at first, then public the same day (JC, 2026-10-15).
	- Progress log:
		- 20261015: public, with secret scanning on. A plain `go get` works with no settings.
	- Closed: 2026-10-15

## Template

### New format

- Notes:

	- Only use rows that you actually need or expect will be filled in. Always fill in the title, ID, Type, Status, Opened and Created by.

	- The ID is the local time to the hundredth of a second. Opened is when it was written down, which may differ. (Use a keyboard macro and possibly something like project 'zuid' to generate.)

	- Status values meaning: Testing means the fix is in and checks are running or still to run. Waiting on signoff means automated testing passed. Moot means something else changed that made it irrelevant. Canceled means it still applies but was decided against. Waiting for testing means the fix is in and waits on a long CI run or an outside test host. Can't reproduce means a real attempt to reproduce it failed.

	- As issues are worked, and statuses change, place them in correct sorting order within the list:
		- First by status: Waiting for answers, Waiting on signoff, Testing, Waiting for testing, Can't reproduce, Stalled, Started, Queued, Done, Deferred, Canceled, Moot
		- Then by severity|priority: Critical, High, Avg, Low
		- Then by type: Bugs, [not bugs together]

	- Rows marked [Bug] are for bugs only, and rows marked [Feature] for features and enhancements. Priority and Severity share one row and one scale. Priority is for a Feature or Enhancement, and Severity for a Bug. Children are not nested. They sit at the top level and point back with Parent ID.

Template:

- Title
	- ID: YYYYmmDDHHMMSSNN
	- Type: [Bug|Feature|Enhancement|Task]
	- Status: [Queued|Waiting for answers|Waiting on signoff|Waiting for testing|Started|Testing|Stalled|Can't reproduce|Moot|Canceled|Deferred|Done]
	- Needs local test suite run?:
	- Needs external testing:
	- Priority [Feature|Enhancement] | Severity [Bug]: [Critical|High|Avg|Low]
	- Opened:
	- Opened by:
	- Assigned to:
	- Parent ID:
	- Prereq IDs:
	- Related IDs:
	- Target OS:
	- Test environment:
	- Version and build:
	- Requirements  [Feature]:
		- Hierarchical bulleted list.
	- Steps to reproduce [Bug]:
		- …
	- Incorrect behavior [Bug]:
	- Expected behavior [Bug]:
	- Reproduced [Bug]: [No, or when, where and how]
	- Possible cause [Bug]:
	- Actual cause [Bug]:
		- …
	- Estimated effort: [High|Avg|Low]
	- Actual effort: [High|Avg|Low]
	- Progress log:
		- …
	- Decisions:
		- …
	- Actual fix [Bug]:
	- Branch:
	- Commit:
	- Test case: [Reason not applicable, or CI test case #]
	- Acceptance signoff:
	- Superseded by ID:
	- Closed:
