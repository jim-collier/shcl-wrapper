// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! The calls the ops scripts can't reach: read-back checks and opening files.

use std::fs;
use std::path::PathBuf;

use shclw::{verify_put, ErrKind, Lib, Value};

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
