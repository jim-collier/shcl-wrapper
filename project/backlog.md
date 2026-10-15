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
	- Test case: tests/cases 001 to 010, Go `api_test.go`, Rust `tests/api.rs`

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

- Layered loading
	- ID: 2026101512024156
	- Type: Feature
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- Defaults, then site, then user file, as one document.
	- Notes:
		- Cobblevale uses shcl's merge. Mossrealm does it by hand.

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

- Tests for upgrade and save failures
	- ID: 2026101512024169
	- Type: Task
	- Status: Queued
	- Priority: Avg
	- Opened: 2026-10-15
	- Opened by: JC
	- Requirements:
		- A refused save, a taken backup name, and an upgrade of a real file, on each version that has them.

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
