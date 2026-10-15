// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! The calls the ops scripts can't reach: read-back checks and opening files.

use std::fs;
use std::path::PathBuf;

use shclw::{verify_put, ErrKind, Lib, ShclDoc, Status, Value};

mod common;
use common::Scratch;

fn check_verify<L: Lib>(n: u32) {
	let d = L::parse("a: 1\n");
	assert!(verify_put(&d, "a", &Value::Int(1)).is_ok(), "v{n}: matching value");
	let e = verify_put(&d, "a", &Value::Int(2)).expect_err("different value");
	assert_eq!(e.kind, ErrKind::NoReadBack, "v{n}: different value");
	let e = verify_put(&d, "nope", &Value::Strs(Vec::new())).expect_err("missing array");
	assert_eq!(e.kind, ErrKind::NoReadBack, "v{n}: missing array");
}

#[test]
fn verify() {
	check_verify::<shclw::v1::Version>(1);
	check_verify::<shclw::v2::Version>(2);
	check_verify::<shclw::v3::Version>(3);
}

/// An f32 has to go in as its own shortest digits. Widened to f64 first,
/// 0.9 was written as 0.8999999761581421.
// The read-back has to be exact.
#[allow(clippy::float_cmp)]
fn check_f32<L: Lib>(n: u32) {
	let texts = [(0.9_f32, "f: 0.9\n"), (0.1, "f: 0.1\n"), (-2.5, "f: -2.5\n")];
	for f in [0.9_f32, 0.1, -2.5, 1e-10, f32::MAX, 16_777_216.0, f32::from_bits(1)] {
		let mut d = L::parse("");
		if let Err(e) = d.put("f", f) {
			panic!("v{n} {f}: {e}");
		}
		let want: f64 = f.to_string().parse().unwrap();
		let g = d.get_float("f");
		assert_eq!((g.status, g.value), (Status::Good, want), "v{n} {f}: read back");
		if let Some((_, text)) = texts.iter().find(|(x, _)| *x == f) {
			assert_eq!(d.text(), *text, "v{n} {f}: text");
		}
	}
	for f in [f32::NAN, f32::INFINITY] {
		let mut d = L::parse("");
		let e = d.put("f", f).expect_err("non-finite put");
		assert_eq!(e.kind, ErrKind::BadValue, "v{n} {f}");
		assert!(!d.has("f"), "v{n} {f}: the put changed the document");
	}
}

#[test]
fn put_f32() {
	check_f32::<shclw::v1::Version>(1);
	check_f32::<shclw::v2::Version>(2);
	check_f32::<shclw::v3::Version>(3);
}

#[test]
fn open() {
	let scratch = Scratch::new("api");
	let dir = scratch.path();
	let newer = dir.join("newer.shcl");
	fs::write(&newer, "a: 1\n##    Format   9\n").unwrap();
	let binary = dir.join("binary.shcl");
	fs::write(&binary, b"a: \xff\xfe\n").unwrap();
	let missing = dir.join("missing.shcl");
	let v3file = dir.join("v3.shcl");
	fs::write(&v3file, "a: [x]\n##    Format   3\n").unwrap();
	let s = |p: &PathBuf| p.to_string_lossy().into_owned();
	let kind = |r: Result<(), shclw::Error>| r.err().map(|e| e.kind);
	for (path, want) in [
		(&newer, ErrKind::TooNew),
		(&binary, ErrKind::Io),
		(&missing, ErrKind::NotFound),
	] {
		assert_eq!(
			kind(shclw::v1::open(&s(path)).map(drop)),
			Some(want),
			"v1 {}",
			path.display()
		);
		assert_eq!(
			kind(shclw::v2::open(&s(path)).map(drop)),
			Some(want),
			"v2 {}",
			path.display()
		);
		assert_eq!(
			kind(shclw::v3::open(&s(path)).map(drop)),
			Some(want),
			"v3 {}",
			path.display()
		);
	}
	assert_eq!(
		kind(shclw::v1::open(&s(&v3file)).map(drop)),
		Some(ErrKind::TooNew),
		"v1 on a format 3 file"
	);
}

/// An edit made through `inner_mut`, with calls the wrapper doesn't have,
/// has to show up in the wrapper's own reads and text.
fn check_inner<D: ShclDoc>(n: u32, d: &D, paths: &[String]) {
	assert!(paths.iter().any(|p| p == "a"), "v{n}: inner().paths() = {paths:?}");
	assert_eq!(d.get_str("a").status, Status::Empty, "v{n}: after set_empty");
	assert_eq!(d.str("b", ""), "set inside", "v{n}: after set_string");
	assert!(d.text().contains("set inside"), "v{n}: text() misses the inner edit");
}

#[test]
fn inner() {
	let mut d1 = shclw::v1::parse("a: 1\n");
	assert!(d1.inner_mut().set_empty("a"), "v1 set_empty");
	assert!(d1.inner_mut().set_string("b", "set inside"), "v1 set_string");
	check_inner(1, &d1, &d1.inner().paths());

	let mut d2 = shclw::v2::parse("a: 1\n");
	assert!(d2.inner_mut().set_empty("a"), "v2 set_empty");
	assert!(d2.inner_mut().set_string("b", "set inside"), "v2 set_string");
	check_inner(2, &d2, &d2.inner().paths());

	let mut d3 = shclw::v3::parse("a: 1\n");
	assert_eq!(
		d3.inner_mut().set_empty("a"),
		shclw::v3::shcl::SetStatus::Ok,
		"v3 set_empty"
	);
	assert_eq!(
		d3.inner_mut().set_string("b", "set inside"),
		shclw::v3::shcl::SetStatus::Ok,
		"v3 set_string"
	);
	check_inner(3, &d3, &d3.inner().paths());
}

/// A 2.x file the old shcl reported errors on is refused, and `upgrade_file`
/// leaves it as it was, with no backup. A clean one still goes.
#[test]
fn upgrade_refuses_old_errors() {
	let scratch = Scratch::new("upgrade");
	let dir = scratch.path().join("c");
	for (text, line) in [("a: 1\nb: ```\nraw line\nc: 2\n", 2), ("x: [ab]\ny: 1\n", 1)] {
		let e = shclw::v3::upgrade(text, 2).expect_err("upgrade");
		assert_eq!(e.kind, ErrKind::Refused, "upgrade {text:?}");
		assert_eq!(e.diags.first().map(|d| d.line), Some(line), "upgrade {text:?}");
		let _ = fs::remove_dir_all(&dir);
		fs::create_dir_all(&dir).unwrap();
		let path = dir.join("c.shcl");
		fs::write(&path, text).unwrap();
		let e = shclw::v3::upgrade_file(&path.to_string_lossy(), 2).expect_err("upgrade_file");
		assert_eq!(e.kind, ErrKind::Refused, "upgrade_file {text:?}");
		assert_eq!(
			fs::read_to_string(&path).unwrap(),
			text,
			"upgrade_file changed the file"
		);
		assert_eq!(
			fs::read_dir(&dir).unwrap().count(),
			1,
			"upgrade_file {text:?}: files beside it"
		);
	}
	let u = shclw::v3::upgrade("tags: a, b\n", 2).expect("clean 2.x file");
	assert!(u.changed, "clean 2.x file: {u:?}");
}

/// A new document has no lines of its own to keep, so it never rewrites. A
/// load at any strictness counts the same as `parse`.
fn check_rewrites_new<L: Lib>(n: u32, mut fresh: L::Doc) {
	fresh.put("a.b", 1_i64).expect("put");
	assert!(!fresh.rewrites(), "v{n}: new document says it rewrites");
	let (loose, _) = L::parse_keep("a:   1\n", shclw::Strictness::Loose);
	assert_eq!(
		loose.rewrites(),
		n < 3,
		"v{n}: loose load of a text that isn't canonical"
	);
}

#[test]
fn rewrites_new() {
	check_rewrites_new::<shclw::v1::Version>(1, shclw::v1::new());
	check_rewrites_new::<shclw::v2::Version>(2, shclw::v2::new());
	check_rewrites_new::<shclw::v3::Version>(3, shclw::v3::new());
}

/// `first` and `last` pick by line, so the fallback reads and an edit take
/// that instance alone. An instance a put made has no line and sorts last.
fn check_first_last<L: Lib>(n: u32) {
	let mut d = L::parse("p: 1\ntv: a\n\tf: 1\ntv: b\n\tf: 2\ntv: a\n\tf: 3\np: 2\n");
	assert_eq!(d.int(&d.last("p"), 9), 2, "v{n}: int of last p");
	assert_eq!(d.str(&d.first("tv.f"), ""), "1", "v{n}: str of first tv.f");
	assert_eq!(d.int(&d.last("tv.f"), 9), 3, "v{n}: int of last tv.f");
	assert_eq!(d.int("p", 9), 9, "v{n}: plain int of p");
	let last = d.last("p");
	d.put(&last, 5).unwrap_or_else(|e| panic!("v{n}: put on last p: {e}"));
	assert_eq!(d.get_int(&d.first("p")).value, 1, "v{n}: first p after the put");
	assert_eq!(d.int(&d.last("p"), 9), 5, "v{n}: last p after the put");
	assert_eq!(d.lines("p"), [1, 8], "v{n}: lines of p after the put");
	let tv = d.each("tv");
	let (a, b) = (d.child(&tv[0].path, "g"), d.child(&tv[1].path, "g"));
	d.put(&b, 2).unwrap_or_else(|e| panic!("v{n}: {e}"));
	d.put(&a, 1).unwrap_or_else(|e| panic!("v{n}: {e}"));
	assert_eq!(d.int(&d.last("tv.g"), 9), 2, "v{n}: last of two put instances");
	assert_eq!(d.lines("tv.g"), [0, 0], "v{n}: lines of two put instances");
	assert_eq!(d.last("nope"), "nope", "v{n}: last of a missing path");
}

#[test]
fn first_last() {
	check_first_last::<shclw::v1::Version>(1);
	check_first_last::<shclw::v2::Version>(2);
	check_first_last::<shclw::v3::Version>(3);
}
