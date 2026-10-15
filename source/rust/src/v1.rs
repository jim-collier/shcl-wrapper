// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! The wrapper over shcl 1.2.
//!
//! shcl 1.2 has no save, no keep-lines text, and no upgrade. `text` gives
//! the canonical form, `write` saves it atomically, and `upgrade` only
//! refuses files newer than 1.2 can read.

use shcl1 as shcl;

use crate::{Diag, ErrKind, Error, Got, Node, Severity, ShclDoc, Status, Strictness, Upgraded, Value};

/// The shcl release this module wraps.
pub const SHCL_VERSION: &str = "1.2.0";

/// The newest file format this shcl reads.
pub const FORMAT_MAJOR: u32 = 1;

/// A parsed document.
#[derive(Debug)]
pub struct Doc {
	d: shcl::Document,
}

/// This module's calls as a [`crate::Lib`].
#[derive(Debug)]
pub struct Version;

impl crate::Lib for Version {
	type Doc = Doc;
	const NAME: &'static str = "shcl 1.2.0";
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
	}
}

/// Never fails. Problems are in `diags`.
pub fn parse(text: &str) -> Doc {
	Doc {
		d: shcl::Document::parse(text),
	}
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
		Ok(d) => (Doc { d }, None),
		Err(le) => {
			let mut e = Error::new(ErrKind::Strict).msg(le.to_string());
			e.diags = diags(&le.diagnostics);
			(Doc { d: le.document }, Some(e))
		}
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

/// Nothing to convert in 1.2. Hands the text back, or refuses a file newer
/// than 1.2 reads.
pub fn upgrade(text: &str, from: u32) -> Result<Upgraded, Error> {
	let f = format_of(text).max(from);
	if f > FORMAT_MAJOR {
		return Err(too_new(f));
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

impl Doc {
	/// The shcl document underneath, for calls shclw doesn't cover.
	pub fn inner(&self) -> &shcl::Document {
		&self.d
	}

	/// The shcl document underneath, to change it with calls shclw doesn't
	/// cover.
	pub fn inner_mut(&mut self) -> &mut shcl::Document {
		&mut self.d
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
		self.d.to_canonical()
	}

	fn canonical(&self) -> String {
		self.d.to_canonical()
	}

	fn write(&self, path: &str) -> Result<(), Error> {
		crate::write_atomic(path, &self.d.to_canonical())
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
		diags(&v)
	}

	fn int(&self, path: &str, fallback: i64) -> i64 {
		self.d.get_int(path).unwrap_or(fallback)
	}

	fn float(&self, path: &str, fallback: f64) -> f64 {
		self.d.get_float(path).unwrap_or(fallback)
	}

	fn bool(&self, path: &str, fallback: bool) -> bool {
		self.d.get_bool(path).unwrap_or(fallback)
	}

	fn str(&self, path: &str, fallback: &str) -> String {
		self.d.get_string(path).unwrap_or_else(|_| fallback.to_owned())
	}

	fn strs(&self, path: &str, fallback: &[&str]) -> Vec<String> {
		self.d
			.get_string_array(path)
			.unwrap_or_else(|_| fallback.iter().map(|s| (*s).to_owned()).collect())
	}

	fn ints(&self, path: &str, fallback: &[i64]) -> Vec<i64> {
		self.d.get_int_array(path).unwrap_or_else(|_| fallback.to_vec())
	}

	fn floats(&self, path: &str, fallback: &[f64]) -> Vec<f64> {
		self.d.get_float_array(path).unwrap_or_else(|_| fallback.to_vec())
	}

	fn raw(&self, path: &str, fallback: &str) -> String {
		self.d.get_raw(path).unwrap_or_else(|_| fallback.to_owned())
	}

	fn get_int(&self, path: &str) -> Got<i64> {
		got(self.d.read_int(path))
	}

	fn get_float(&self, path: &str) -> Got<f64> {
		got(self.d.read_float(path))
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
		got(self.d.read_float_array(path))
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
		let n = self.d.count(path);
		let vals = self.d.instances(path);
		(0..n)
			.map(|i| {
				let p = if n > 1 {
					format!("{path}[#{i}]")
				} else {
					path.to_owned()
				};
				let value = vals.get(i).cloned().unwrap_or_default();
				let line = self.d.line(&p);
				Node { path: p, value, line }
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
		if self.d.count(path) > 1 {
			return Err(Error::new(ErrKind::BadPath).path(path).msg("Multiple"));
		}
		let v = v.into();
		let ok = match &v {
			Value::Int(x) => self.d.set_int(path, *x),
			Value::Float(x) => self.d.set_float(path, *x),
			Value::Bool(x) => self.d.set_bool(path, *x),
			Value::Str(x) => self.d.set_string(path, x),
			Value::Strs(x) => {
				let refs: Vec<&str> = x.iter().map(String::as_str).collect();
				self.d.set_string_array(path, &refs)
			}
			Value::Ints(x) => self.d.set_int_array(path, x),
			Value::Floats(x) => self.d.set_float_array(path, x),
		};
		self.set_err(path, ok)?;
		crate::verify_put(self, path, &v)
	}

	fn put_syntax(&mut self, path: &str, text: &str) -> Result<(), Error> {
		let ok = self.d.set_literal(path, text);
		self.set_err(path, ok)
	}

	fn comment(&mut self, path: &str, text: &str) -> Result<(), Error> {
		let ok = self.d.set_comment(path, text);
		self.set_err(path, ok)
	}

	fn remove(&mut self, path: &str) -> usize {
		self.d.remove(path)
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
