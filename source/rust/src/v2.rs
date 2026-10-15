// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! The wrapper over shcl 2.0.
//!
//! shcl 2.0 has no keep-lines text and no upgrade. `text` gives the
//! canonical form, `write` refuses with `Refused` when the save would drop
//! lines the load couldn't use, and `upgrade` only refuses files 2.0 can't
//! read.

/// The shcl crate underneath, so code using `inner()` can name its types.
pub use shcl2 as shcl;

use crate::{Diag, ErrKind, Error, Got, Node, Severity, ShclDoc, Status, Strictness, Upgraded, Value};

/// The shcl release this module wraps.
pub const SHCL_VERSION: &str = "2.0.0";

/// The newest file format this shcl reads.
pub const FORMAT_MAJOR: u32 = 2;

/// A parsed document.
#[derive(Debug)]
pub struct Doc {
	d: shcl::Document,
	/// The loaded text wasn't canonical, so a save changes its lines.
	rewrites: bool,
	info: crate::Info,
	stamp: bool,
}

/// This module's calls as a [`crate::Lib`].
#[derive(Debug)]
pub struct Version;

impl crate::Lib for Version {
	type Doc = Doc;
	const NAME: &'static str = "shcl 2.0.0";
	const FORMAT_MAJOR: u32 = FORMAT_MAJOR;
	fn parse(text: &str) -> Doc {
		parse(text)
	}
	fn parse_keep(text: &str, level: Strictness) -> (Doc, Option<Error>) {
		parse_keep(text, level)
	}
	fn format_of(text: &str) -> u32 {
		format_of(text)
	}
	fn upgrade(text: &str, from: u32) -> Result<Upgraded, Error> {
		upgrade(text, from)
	}
}

/// An empty document.
pub fn new() -> Doc {
	Doc {
		d: shcl::Document::new(),
		rewrites: false,
		info: crate::Info::AsIs,
		stamp: false,
	}
}

/// Never fails. Problems are in `diags`.
pub fn parse(text: &str) -> Doc {
	loaded(shcl::Document::parse(text), text)
}

/// Fails only at `Strict`, with `ErrKind::Strict`.
pub fn parse_with(text: &str, level: Strictness) -> Result<Doc, Error> {
	match parse_keep(text, level) {
		(d, None) => Ok(d),
		(_, Some(e)) => Err(e),
	}
}

fn parse_keep(text: &str, level: Strictness) -> (Doc, Option<Error>) {
	match shcl::Document::parse_with(text, strictness(level)) {
		Ok(d) => (loaded(d, text), None),
		Err(le) => {
			let mut e = Error::new(ErrKind::Strict).msg(le.to_string());
			e.diags = diags(&le.diagnostics);
			(loaded(le.document, text), Some(e))
		}
	}
}

fn loaded(d: shcl::Document, text: &str) -> Doc {
	let rewrites = d.to_canonical() != text;
	Doc {
		d,
		rewrites,
		info: crate::Info::AsIs,
		stamp: false,
	}
}

/// Reads and parses a file. A missing file is `NotFound`, and a file
/// written for a newer format is `TooNew`.
pub fn open(path: &str) -> Result<Doc, Error> {
	open_with(path, Strictness::Standard)
}

/// `open` at a given strictness.
pub fn open_with(path: &str, level: Strictness) -> Result<Doc, Error> {
	let text = crate::read_text(path)?;
	let f = format_of(&text);
	if f > FORMAT_MAJOR {
		return Err(too_new(f).path(path));
	}
	parse_with(&text, level).map_err(|e| e.path(path))
}

/// The format a file's Format line names, or 0 with none.
pub fn format_of(text: &str) -> u32 {
	crate::scan_format(text)
}

/// Nothing to convert in 2.0. Hands the text back, refuses a file newer
/// than 2.0 reads, and can't take a 1.x file forward.
pub fn upgrade(text: &str, from: u32) -> Result<Upgraded, Error> {
	let f = format_of(text).max(from);
	if f > FORMAT_MAJOR {
		return Err(too_new(f));
	}
	if from != 0 && from < FORMAT_MAJOR {
		return Err(Error::new(ErrKind::Unsupported).msg(format!("shcl {SHCL_VERSION} can't upgrade format {from}")));
	}
	Ok(Upgraded {
		text: text.to_owned(),
		from: FORMAT_MAJOR,
		..Upgraded::default()
	})
}

/// `upgrade` on a file. It never writes.
pub fn upgrade_file(path: &str, from: u32) -> Result<Upgraded, Error> {
	let text = crate::read_text(path)?;
	upgrade(&text, from).map_err(|e| e.path(path))
}

fn info_edit() -> crate::InfoEdit<'static> {
	static BLOCK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
	crate::InfoEdit {
		block: BLOCK.get_or_init(|| crate::info_text(FORMAT_MAJOR, "v2.0.0")),
		major: FORMAT_MAJOR,
		values,
	}
}

/// Every path's values and the load's error codes, for `crate::finish_info`
/// to tell a comment from a raw body's line.
fn values(text: &str) -> String {
	let d = shcl::Document::parse(text);
	let mut s = String::new();
	for p in d.paths() {
		s.push_str(&p);
		for v in d.instances(&p) {
			s.push('\0');
			s.push_str(&v);
		}
		s.push('\x01');
	}
	for x in d.diagnostics() {
		if x.severity == shcl::Severity::Error {
			s.push_str(x.code);
			s.push('\x02');
		}
	}
	s
}

impl Doc {
	/// The save's text, with the info block and the Format line as
	/// `info_block` and `stamp_format` asked.
	fn saved(&self, path: &str) -> Result<String, Error> {
		crate::finish_info(&self.d.to_canonical(), self.info, self.stamp, &info_edit()).map_err(|e| e.path(path))
	}

	/// `saved`, or the text without the block and the line when an edit since
	/// made them impossible, which `write` reports.
	fn finish(&self) -> String {
		self.saved("").unwrap_or_else(|_| self.d.to_canonical())
	}

	/// The shcl document underneath, for calls shclw doesn't cover.
	pub fn inner(&self) -> &shcl::Document {
		&self.d
	}

	/// The shcl document underneath, to change it with calls shclw doesn't
	/// cover.
	pub fn inner_mut(&mut self) -> &mut shcl::Document {
		&mut self.d
	}

	/// The one way an edit reaches the shcl document, so none can skip the
	/// check that the path names one instance at every step. shcl 2.0
	/// would edit the first one and say ok, where 3.0 refuses.
	fn set(&mut self, path: &str, apply: impl FnOnce(&mut shcl::Document) -> bool) -> Result<(), Error> {
		crate::one_target(self, path)?;
		let ok = apply(&mut self.d);
		self.set_err(path, ok)
	}

	fn set_err(&self, path: &str, ok: bool) -> Result<(), Error> {
		if ok {
			return Ok(());
		}
		let r = self.d.write_reason(path);
		if r == shcl::WriteReason::Writable {
			return Err(Error::new(ErrKind::BadValue).path(path));
		}
		Err(Error::new(ErrKind::BadPath).path(path).msg(format!("{r:?}")))
	}
}

impl ShclDoc for Doc {
	fn text(&self) -> String {
		self.finish()
	}

	fn canonical(&self) -> String {
		self.finish()
	}

	/// Refuses when the load dropped lines the save would lose, the same
	/// check shcl's `save_file` makes, and writes nothing when the info block
	/// or the Format line can't go in, since an edit made after `info_block`
	/// or `stamp_format`.
	fn write(&self, path: &str) -> Result<(), Error> {
		let lost = self.d.lost_count();
		if lost > 0 {
			return Err(Error::new(ErrKind::Refused)
				.path(path)
				.msg(format!("{lost} lines would be lost")));
		}
		shcl::write_file_atomic(path, &self.saved(path)?).map_err(|m| Error::new(ErrKind::Io).path(path).msg(m))
	}

	/// 2.0 only saves canonical form, so this is any loaded text that
	/// wasn't canonical already. A `new` document has no lines to keep.
	/// `info_block` and `stamp_format` change only their own lines, so they
	/// don't change the answer.
	fn rewrites(&self) -> bool {
		self.rewrites
	}

	fn diags(&self) -> Vec<Diag> {
		diags(self.d.diagnostics())
	}

	fn errors(&self) -> usize {
		self.d.error_count()
	}

	fn check(&self, schema: &Doc) -> Vec<Diag> {
		diags(&self.d.validate(&schema.d))
	}

	fn drop_declared(&self, ds: Vec<Diag>) -> Vec<Diag> {
		let mut v = undiags(&ds);
		shcl::suppress_declared_repeats(&self.d, &mut v);
		shcl::suppress_declared_reopens(&self.d, &mut v);
		diags(&v)
	}

	fn int(&self, path: &str, fallback: i64) -> i64 {
		self.d.get_int_or(path, fallback)
	}

	fn float(&self, path: &str, fallback: f64) -> f64 {
		// Through get_float, so infinity gets the fallback too.
		let g = self.get_float(path);
		if g.ok() {
			g.value
		} else {
			fallback
		}
	}

	fn bool(&self, path: &str, fallback: bool) -> bool {
		self.d.get_bool_or(path, fallback)
	}

	fn str(&self, path: &str, fallback: &str) -> String {
		self.d.get_string_or(path, fallback.to_owned())
	}

	fn strs(&self, path: &str, fallback: &[&str]) -> Vec<String> {
		self.d
			.get_string_array_or(path, fallback.iter().map(|s| (*s).to_owned()).collect())
	}

	fn ints(&self, path: &str, fallback: &[i64]) -> Vec<i64> {
		self.d.get_int_array_or(path, fallback.to_vec())
	}

	fn floats(&self, path: &str, fallback: &[f64]) -> Vec<f64> {
		let g = self.get_floats(path);
		if g.ok() {
			g.value
		} else {
			fallback.to_vec()
		}
	}

	fn raw(&self, path: &str, fallback: &str) -> String {
		self.d.get_raw_or(path, fallback.to_owned())
	}

	fn get_int(&self, path: &str) -> Got<i64> {
		got(self.d.read_int(path))
	}

	fn get_float(&self, path: &str) -> Got<f64> {
		crate::finite_float(got(self.d.read_float(path)))
	}

	fn get_bool(&self, path: &str) -> Got<bool> {
		got(self.d.read_bool(path))
	}

	fn get_str(&self, path: &str) -> Got<String> {
		got(self.d.read_string(path))
	}

	fn get_strs(&self, path: &str) -> Got<Vec<String>> {
		got(self.d.read_string_array(path))
	}

	fn get_ints(&self, path: &str) -> Got<Vec<i64>> {
		got(self.d.read_int_array(path))
	}

	fn get_floats(&self, path: &str) -> Got<Vec<f64>> {
		crate::finite_floats(got(self.d.read_float_array(path)))
	}

	fn get_raw(&self, path: &str) -> Got<String> {
		got(self.d.read_raw(path))
	}

	fn has(&self, path: &str) -> bool {
		self.d.exists(path)
	}

	fn count(&self, path: &str) -> usize {
		self.d.count(path)
	}

	fn names(&self, path: &str) -> Vec<String> {
		if path.is_empty() || self.d.count(path) <= 1 {
			return crate::dedupe_names(self.d.children(path));
		}
		let all = self.each(path).iter().flat_map(|n| self.d.children(&n.path)).collect();
		crate::dedupe_names(all)
	}

	fn each(&self, path: &str) -> Vec<Node> {
		crate::each_paths(self, |p| self.d.children(p), path)
			.into_iter()
			.map(|p| Node {
				value: self.d.instances(&p).into_iter().next().unwrap_or_default(),
				line: self.d.line(&p),
				path: p,
			})
			.collect()
	}

	fn line(&self, path: &str) -> usize {
		self.d.line(path)
	}

	fn child(&self, base: &str, name: &str) -> String {
		if base.is_empty() {
			return shcl::quote_segment(name);
		}
		format!("{base}.{}", shcl::quote_segment(name))
	}

	fn put(&mut self, path: &str, v: impl Into<Value>) -> Result<(), Error> {
		let v = v.into();
		crate::check_value(path, &v)?;
		self.set(path, |d| match &v {
			Value::Int(x) => d.set_int(path, *x),
			Value::Float(x) => d.set_float(path, *x),
			Value::Bool(x) => d.set_bool(path, *x),
			Value::Str(x) => d.set_string(path, x),
			Value::Strs(x) => {
				let refs: Vec<&str> = x.iter().map(String::as_str).collect();
				d.set_string_array(path, &refs)
			}
			Value::Ints(x) => d.set_int_array(path, x),
			Value::Floats(x) => d.set_float_array(path, x),
		})?;
		crate::verify_put(self, path, &v)
	}

	fn put_syntax(&mut self, path: &str, text: &str) -> Result<(), Error> {
		self.set(path, |d| d.set_literal(path, text))?;
		let mut alone = new();
		// A failed set leaves alone empty, which the check reports.
		let _ = alone.d.set_literal("k", text);
		crate::verify_syntax(self, path, &alone, &parse(&format!("k: {text}\n")))
	}

	fn comment(&mut self, path: &str, text: &str) -> Result<(), Error> {
		self.set(path, |d| d.set_comment(path, text))
	}

	fn remove(&mut self, path: &str) -> usize {
		self.d.remove(path)
	}

	fn info_block(&mut self, on: bool) -> Result<(), Error> {
		let info = if on { crate::Info::On } else { crate::Info::Off };
		crate::finish_info(&self.d.to_canonical(), info, self.stamp, &info_edit())?;
		self.info = info;
		Ok(())
	}

	fn stamp_format(&mut self) -> Result<(), Error> {
		crate::finish_info(&self.d.to_canonical(), self.info, true, &info_edit())?;
		self.stamp = true;
		Ok(())
	}
}

fn too_new(f: u32) -> Error {
	Error::new(ErrKind::TooNew).msg(format!(
		"format {f}, and shcl {SHCL_VERSION} reads up to {FORMAT_MAJOR}"
	))
}

fn strictness(s: Strictness) -> shcl::Strictness {
	match s {
		Strictness::Loose => shcl::Strictness::Loose,
		Strictness::Standard => shcl::Strictness::Standard,
		Strictness::Strict => shcl::Strictness::Strict,
	}
}

fn status(s: shcl::Status) -> Status {
	match s {
		shcl::Status::Good => Status::Good,
		shcl::Status::Empty => Status::Empty,
		shcl::Status::NotFound => Status::NotFound,
		shcl::Status::BadType => Status::BadType,
		shcl::Status::Multiple => Status::Multiple,
	}
}

fn got<T>(r: shcl::Read<T>) -> Got<T> {
	Got {
		value: r.value,
		status: status(r.status),
		line: r.line,
		text: r.raw.unwrap_or_default(),
	}
}

fn diags(v: &[shcl::Diagnostic]) -> Vec<Diag> {
	v.iter()
		.map(|d| Diag {
			line: d.line,
			severity: match d.severity {
				shcl::Severity::Error => Severity::Error,
				shcl::Severity::Hint => Severity::Hint,
			},
			code: d.code,
			message: d.message.clone(),
		})
		.collect()
}

fn undiags(v: &[Diag]) -> Vec<shcl::Diagnostic> {
	v.iter()
		.map(|d| shcl::Diagnostic {
			line: d.line,
			severity: match d.severity {
				Severity::Error => shcl::Severity::Error,
				Severity::Hint => shcl::Severity::Hint,
			},
			code: d.code,
			message: d.message.clone(),
		})
		.collect()
}
