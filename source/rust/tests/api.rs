// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! The calls the ops scripts can't reach: read-back checks and opening files.

use std::fs;
use std::path::PathBuf;

use shclw::{verify_put, ErrKind, Lib, ShclDoc, Status, Value};

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

fn scratch() -> PathBuf {
	let base = std::env::var_os("TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
	let dir = base.join(format!("shclw-rust-api-{}", std::process::id()));
	fs::create_dir_all(&dir).unwrap();
	dir
}

#[test]
fn open() {
	let dir = scratch();
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
	let _ = fs::remove_dir_all(&dir); // only this test's own dir
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
