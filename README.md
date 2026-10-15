<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Lifecycle: Alpha](https://img.shields.io/badge/Lifecycle-Alpha-orange)
![Support](https://img.shields.io/badge/Support-Maintained-brightgreen)
![Go](https://img.shields.io/badge/Go-00ADD8?logo=go&logoColor=white)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

</div>

<!-- TOC ignore:true -->
# shcl-wrapper

One small, steady API over every version of [shcl](https://github.com/yottacore/shcl), in each language that uses it.

<!-- TOC ignore:true -->
## Table of contents

<!-- TOC -->

- [Why](#why)
- [Features](#features)
- [Using it](#using-it)
	- [Go](#go)
	- [Rust](#rust)
- [Building from source](#building-from-source)
- [Support](#support)
- [Copyright and license](#copyright-and-license)

<!-- /TOC -->

## Why

shcl is moving fast. 1.2, 2.0, and 3.0 came out within a few months of each other, 3.0 changes the file format a little, and 4 is set to change the API a lot. A program that calls shcl directly has to follow every one of those changes, and each program ends up with its own fixes for the same quirks.

This wraps the basics, loading, reading, walking, writing, saving, and upgrading, in one API that stays put. A program picks its shcl version in one place. When shcl changes, the wrapper changes, once, with tests, and every program using it gets the fix.

If your program uses one shcl version and doesn't plan to move, you don't need this. Call shcl directly. And if it leans on shcl's deeper features, like line-keeping merges or schema generation, the wrapper won't cover those. It only gets out of the way, with `Inner()`.

## Features

- The same calls in Go and Rust, over shcl 1.2, 2.0, and 3.0.

- Reads with a fallback, `doc.Int("server.port", 8080)`, or with a status when the reason matters.

- Walking repeated entries without writing selector syntax, which changed between versions.

- Every write is read back. A value shcl stored differently than it was given comes back as an error.

- A file written for a newer format is refused with a clear error.

- Format upgrades with a backup of the original, where the shcl underneath can do one.

- A text script of edits and reads, which is also what the tests run in every language, so Go and Rust can't drift apart.

## Using it

### Go

~~~bash
go get github.com/jim-collier/shcl-wrapper/source/go@dev
~~~

~~~go
import shclw "github.com/jim-collier/shcl-wrapper/source/go/shclw3"

doc, err := shclw.Open("app.shcl")
if err != nil && shclw.KindOf(err) != shclw.ErrNotFound {
	return err
}
port := doc.Int("server.port", 8080)
for _, t := range doc.Each("table") {
	fmt.Println(doc.Str(doc.Child(t.Path, "name"), ""))
}
if err := doc.Put("server.port", 9090); err != nil {
	return err
}
return doc.Write("app.shcl")
~~~

`shclw1`, `shclw2`, and `shclw3` match shcl 1.2, 2.0, and 3.0. Moving to another version is the import line.

### Rust

~~~toml
[dependencies]
shclw = { git = "https://github.com/jim-collier/shcl-wrapper", branch = "dev", features = ["shcl3"] }
~~~

~~~rust
use shclw::ShclDoc;

let mut doc = shclw::v3::open("app.shcl")?;
let port = doc.int("server.port", 8080);
for t in doc.each("table") {
	println!("{}", doc.str(&doc.child(&t.path, "name"), ""));
}
doc.put("server.port", 9090)?;
doc.write("app.shcl")?;
~~~

The features `shcl1`, `shcl2`, and `shcl3` turn on the modules `v1`, `v2`, and `v3`.

The full list of calls, and what the wrapper does differently from each shcl version, is in [design.md](project/design.md).

## Building from source

Go 1.21 or newer, and Rust 1.88 or newer. The pipeline pins Rust 1.96 in `source/rust/rust-toolchain.toml`.

~~~bash
cicd/cicd.bash           # format, lint, tests, and fuzzing
cicd/cicd.bash --gate    # the same without fuzzing, as the pre-push hook runs it
~~~

## Support

If it saves you some time, you can say thanks through [GitHub Sponsors](https://github.com/sponsors/jim-collier) or [Ko-fi](https://ko-fi.com/jimcollier).

## Copyright and license

> Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]<br />
> Licensed under the [MIT License](https://mit-license.org/)<br />
> SPDX-License-Identifier: `MIT`<br />
> No warranty.<br />
> shcl-wrapper™ is a [trademark](trademark.md) of Jim Collier.
