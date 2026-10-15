// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

//! A small wrapper over several versions of shcl.
//!
//! Turn on the cargo feature for the shcl the program builds against,
//! `shcl1`, `shcl2`, or `shcl3`, and use that module. Every module has the
//! same calls, through the [`ShclDoc`] trait, so moving to another shcl
//! version is a change to the feature and the module name.
//!
//! ```ignore
//! use shclw::ShclDoc;
//! let doc = shclw::v3::open("app.shcl")?;
//! let port = doc.int("server.port", 8080);
//! ```

// Every call returns the one Error type, whose kinds are documented there,
// so a per-function errors section would say the same thing 30 times.
#![allow(clippy::missing_errors_doc)]
// Most calls are reads, and nothing is gained by marking each one.
#![allow(clippy::must_use_candidate)]

use std::fmt::{self, Write as _};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// How a read went. `BadPath` is new in shcl 3.0; older versions say
/// `NotFound` for a path that doesn't parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
	Good,
	Empty,
	NotFound,
	BadType,
	Multiple,
	BadPath,
}

impl fmt::Display for Status {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Status::Good => "Good",
			Status::Empty => "Empty",
			Status::NotFound => "NotFound",
			Status::BadType => "BadType",
			Status::Multiple => "Multiple",
			Status::BadPath => "BadPath",
		})
	}
}

/// How forgiving a load is. Only `Strict` can fail one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Strictness {
	Loose,
	#[default]
	Standard,
	Strict,
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
	Error,
	Hint,
}

impl fmt::Display for Severity {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Severity::Error => "error",
			Severity::Hint => "hint",
		})
	}
}

/// One diagnostic from a load or a schema check. `line` is 1-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
	pub line: usize,
	pub severity: Severity,
	pub code: &'static str,
	pub message: String,
}

impl fmt::Display for Diag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "line {}: {} {}", self.line, self.code, self.message)
	}
}

/// A read with its status. Check `status` before trusting `value`. `text`
/// is the value as typed in the file, when there was one.
#[derive(Debug, Clone, PartialEq)]
pub struct Got<T> {
	pub value: T,
	pub status: Status,
	pub line: usize,
	pub text: String,
}

impl<T> Got<T> {
	/// True when the read found a usable value.
	pub fn ok(&self) -> bool {
		self.status == Status::Good
	}
}

/// One instance found by `each`. `path` reads this instance and no other;
/// add child names to it with `child`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
	pub path: String,
	pub value: String,
	pub line: usize,
}

/// What went wrong, sorted into what a caller can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrKind {
	NotFound,
	Io,
	Strict,
	Refused,
	BadPath,
	BadValue,
	TooNew,
	Unsupported,
	Ambiguous,
	BackupTaken,
	NoReadBack,
}

impl fmt::Display for ErrKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			ErrKind::NotFound => "notfound",
			ErrKind::Io => "io",
			ErrKind::Strict => "strict",
			ErrKind::Refused => "refused",
			ErrKind::BadPath => "badpath",
			ErrKind::BadValue => "badvalue",
			ErrKind::TooNew => "toonew",
			ErrKind::Unsupported => "unsupported",
			ErrKind::Ambiguous => "ambiguous",
			ErrKind::BackupTaken => "backuptaken",
			ErrKind::NoReadBack => "noreadback",
		})
	}
}

/// The one error type every module returns. `diags` is set for a failed
/// strict load.
#[derive(Debug, Clone)]
pub struct Error {
	pub kind: ErrKind,
	pub path: String,
	pub msg: String,
	pub diags: Vec<Diag>,
}

impl Error {
	pub fn new(kind: ErrKind) -> Error {
		Error {
			kind,
			path: String::new(),
			msg: String::new(),
			diags: Vec::new(),
		}
	}

	#[must_use]
	pub fn path(mut self, path: &str) -> Error {
		path.clone_into(&mut self.path);
		self
	}

	#[must_use]
	pub fn msg(mut self, msg: impl Into<String>) -> Error {
		self.msg = msg.into();
		self
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "shclw: {}", self.kind)?;
		if !self.path.is_empty() {
			write!(f, " {:?}", self.path)?;
		}
		if !self.msg.is_empty() {
			write!(f, ": {}", self.msg)?;
		}
		Ok(())
	}
}

impl std::error::Error for Error {}

/// What an upgrade made of a file. `from` is the format it was written for,
/// and `unsure` counts lines it couldn't convert for sure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Upgraded {
	pub text: String,
	pub from: u32,
	pub changed: bool,
	pub unsure: usize,
	pub backup: String,
}

/// A value for `put`, with the Rust type folded down to the few shcl takes.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Int(i64),
	Float(f64),
	Bool(bool),
	Str(String),
	Strs(Vec<String>),
	Ints(Vec<i64>),
	Floats(Vec<f64>),
}

impl From<i64> for Value {
	fn from(v: i64) -> Value {
		Value::Int(v)
	}
}
impl From<i32> for Value {
	fn from(v: i32) -> Value {
		Value::Int(v.into())
	}
}
impl From<u32> for Value {
	fn from(v: u32) -> Value {
		Value::Int(v.into())
	}
}
impl From<f64> for Value {
	fn from(v: f64) -> Value {
		Value::Float(v)
	}
}
impl From<f32> for Value {
	fn from(v: f32) -> Value {
		Value::Float(v.into())
	}
}
impl From<bool> for Value {
	fn from(v: bool) -> Value {
		Value::Bool(v)
	}
}
impl From<&str> for Value {
	fn from(v: &str) -> Value {
		Value::Str(v.to_owned())
	}
}
impl From<String> for Value {
	fn from(v: String) -> Value {
		Value::Str(v)
	}
}
impl From<Vec<String>> for Value {
	fn from(v: Vec<String>) -> Value {
		Value::Strs(v)
	}
}
impl From<&[&str]> for Value {
	fn from(v: &[&str]) -> Value {
		Value::Strs(v.iter().map(|s| (*s).to_owned()).collect())
	}
}
impl From<Vec<i64>> for Value {
	fn from(v: Vec<i64>) -> Value {
		Value::Ints(v)
	}
}
impl From<&[i64]> for Value {
	fn from(v: &[i64]) -> Value {
		Value::Ints(v.to_vec())
	}
}
impl From<Vec<f64>> for Value {
	fn from(v: Vec<f64>) -> Value {
		Value::Floats(v)
	}
}
impl From<&[f64]> for Value {
	fn from(v: &[f64]) -> Value {
		Value::Floats(v.to_vec())
	}
}

/// What every module's document does. Bring it into scope to call the
/// methods: `use shclw::ShclDoc;`.
pub trait ShclDoc {
	/// The document as it would be saved.
	fn text(&self) -> String;
	/// The document in canonical form.
	fn canonical(&self) -> String;
	/// Saves `text()` atomically.
	fn write(&self, path: &str) -> Result<(), Error>;
	/// What the load found.
	fn diags(&self) -> Vec<Diag>;
	/// Counts error diagnostics.
	fn errors(&self) -> usize;
	/// Validates this document against a schema document.
	fn check(&self, schema: &Self) -> Vec<Diag>;
	/// Takes out the hints for keys this schema says repeat.
	fn drop_declared(&self, diags: Vec<Diag>) -> Vec<Diag>;

	fn int(&self, path: &str, fallback: i64) -> i64;
	fn float(&self, path: &str, fallback: f64) -> f64;
	fn bool(&self, path: &str, fallback: bool) -> bool;
	fn str(&self, path: &str, fallback: &str) -> String;
	fn strs(&self, path: &str, fallback: &[&str]) -> Vec<String>;
	fn ints(&self, path: &str, fallback: &[i64]) -> Vec<i64>;
	fn floats(&self, path: &str, fallback: &[f64]) -> Vec<f64>;
	fn raw(&self, path: &str, fallback: &str) -> String;

	fn get_int(&self, path: &str) -> Got<i64>;
	fn get_float(&self, path: &str) -> Got<f64>;
	fn get_bool(&self, path: &str) -> Got<bool>;
	fn get_str(&self, path: &str) -> Got<String>;
	fn get_strs(&self, path: &str) -> Got<Vec<String>>;
	fn get_ints(&self, path: &str) -> Got<Vec<i64>>;
	fn get_floats(&self, path: &str) -> Got<Vec<f64>>;
	fn get_raw(&self, path: &str) -> Got<String>;

	fn has(&self, path: &str) -> bool;
	fn count(&self, path: &str) -> usize;
	/// Child names under `path` once each, in file order. With several
	/// instances it's every instance's children.
	fn names(&self, path: &str) -> Vec<String>;
	/// Every instance `path` names, each with a path that reads it alone.
	fn each(&self, path: &str) -> Vec<Node>;
	fn line(&self, path: &str) -> usize;
	/// `base` with one more name on the end, quoted if it needs it.
	fn child(&self, base: &str, name: &str) -> String;

	fn put(&mut self, path: &str, v: impl Into<Value>) -> Result<(), Error>;
	/// Sets `path` to `text` read as shcl value syntax.
	fn put_syntax(&mut self, path: &str, text: &str) -> Result<(), Error>;
	/// Sets the comment above `path`.
	fn comment(&mut self, path: &str, text: &str) -> Result<(), Error>;
	/// Takes out `path` and everything under it, and says how many
	/// instances went.
	fn remove(&mut self, path: &str) -> usize;
}

/// One module's package-level calls, for code that picks a shcl version at
/// run time or works with any of them.
pub trait Lib {
	type Doc: ShclDoc;
	const NAME: &'static str;
	const FORMAT_MAJOR: u32;
	fn parse(text: &str) -> Self::Doc;
	/// A load at `level`, with the document back even when it fails.
	fn parse_keep(text: &str, level: Strictness) -> (Self::Doc, Option<Error>);
	fn format_of(text: &str) -> u32;
	fn upgrade(text: &str, from: u32) -> Result<Upgraded, Error>;
}

/// Reads back what a setter just wrote. shcl can accept a value and then
/// read something else, such as a one-item array of "" coming back empty, or
/// a path with two instances reading as `Multiple`. The edit stays either
/// way; `NoReadBack` says it didn't take. An empty array reading as `Empty`
/// is accepted, since 1.2 and 2.0 have no way to write one.
// A float has to come back exactly, so no margin.
#[allow(clippy::float_cmp)]
pub fn verify_put<D: ShclDoc + ?Sized>(d: &D, path: &str, v: &Value) -> Result<(), Error> {
	let (status, same, empty) = match v {
		Value::Int(x) => {
			let g = d.get_int(path);
			(g.status, g.value == *x, false)
		}
		Value::Float(x) => {
			let g = d.get_float(path);
			(g.status, g.value == *x, false)
		}
		Value::Bool(x) => {
			let g = d.get_bool(path);
			(g.status, g.value == *x, false)
		}
		Value::Str(x) => {
			let g = d.get_str(path);
			(g.status, g.value == *x, false)
		}
		Value::Strs(x) => {
			let g = d.get_strs(path);
			(g.status, g.value == *x, x.is_empty())
		}
		Value::Ints(x) => {
			let g = d.get_ints(path);
			(g.status, g.value == *x, x.is_empty())
		}
		Value::Floats(x) => {
			let g = d.get_floats(path);
			(g.status, g.value == *x, x.is_empty())
		}
	};
	if status == Status::Good && same || status == Status::Empty && empty {
		return Ok(());
	}
	let msg = if status == Status::Good {
		"reads back as a different value".to_owned()
	} else {
		format!("reads back as {status}")
	};
	Err(Error::new(ErrKind::NoReadBack).path(path).msg(msg))
}

/// Keeps the first of each name, compared without ASCII case, since shcl
/// matches names that way and lists one per line it was given on.
pub fn dedupe_names(names: Vec<String>) -> Vec<String> {
	let mut seen = std::collections::HashSet::new();
	names
		.into_iter()
		.filter(|n| seen.insert(n.to_ascii_lowercase()))
		.collect()
}

const FORMAT_HEAD: &str = "##    Format   ";

/// The number a `##    Format   N` line names, or 0. It's for modules whose
/// shcl predates the Format line, so they can refuse a newer file rather
/// than misread it.
pub fn scan_format(text: &str) -> u32 {
	for line in text.split('\n') {
		let Some(rest) = line.trim_end_matches('\r').strip_prefix(FORMAT_HEAD) else {
			continue;
		};
		if let Ok(n) = rest.trim().parse::<u32>() {
			if n > 0 {
				return n;
			}
		}
	}
	0
}

/// Reads a whole file, with a missing file as `NotFound`.
pub fn read_text(path: &str) -> Result<String, Error> {
	match fs::read(path) {
		Ok(b) => String::from_utf8(b).map_err(|_| Error::new(ErrKind::Io).path(path).msg("not valid UTF-8")),
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::new(ErrKind::NotFound).path(path)),
		Err(e) => Err(Error::new(ErrKind::Io).path(path).msg(e.to_string())),
	}
}

/// Writes `data` beside `path` and renames it into place, so a reader sees
/// the old file or the new one, never half of one. An existing file's
/// permissions are kept.
pub fn write_atomic(path: &str, data: &str) -> Result<(), Error> {
	let io = |e: std::io::Error| Error::new(ErrKind::Io).path(path).msg(e.to_string());
	let target = Path::new(path);
	let old = match fs::metadata(target) {
		Ok(m) if !m.is_file() => return Err(Error::new(ErrKind::Io).path(path).msg("not a regular file")),
		Ok(m) => Some(m.permissions()),
		Err(_) => None,
	};
	let dir = target
		.parent()
		.filter(|p| !p.as_os_str().is_empty())
		.unwrap_or(Path::new("."));
	let base = target
		.file_name()
		.map(|n| n.to_string_lossy().into_owned())
		.unwrap_or_default();
	let tmp = dir.join(format!(".{base}.tmp{}", std::process::id()));
	let result = (|| {
		let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
		f.write_all(data.as_bytes())?;
		if let Some(p) = &old {
			f.set_permissions(p.clone())?;
		}
		f.sync_all()?;
		drop(f);
		fs::rename(&tmp, target)
	})();
	if let Err(e) = result {
		let _ = fs::remove_file(&tmp); // best effort; the write already failed
		return Err(io(e));
	}
	Ok(())
}

/// Writes a float the same way in every wrapper language: the shortest
/// digits that read back exactly, positional between 1e-5 and 1e17, else
/// with an exponent.
pub fn format_float(f: f64) -> String {
	if f.is_nan() {
		return "nan".to_owned();
	}
	if f.is_infinite() {
		return if f > 0.0 { "inf" } else { "-inf" }.to_owned();
	}
	let s = format!("{f:e}");
	let (mant, exp) = s.split_once('e').unwrap_or((s.as_str(), "0"));
	let exp: i32 = exp.parse().unwrap_or(0);
	let neg = mant.starts_with('-');
	let digits: String = mant.trim_start_matches('-').chars().filter(|c| *c != '.').collect();
	assemble_float(neg, &digits, exp)
}

fn assemble_float(neg: bool, digits: &str, exp: i32) -> String {
	let mut b = String::new();
	if neg {
		b.push('-');
	}
	let len = i32::try_from(digits.len()).unwrap_or(i32::MAX);
	if !(-5..17).contains(&exp) {
		b.push_str(&digits[..1]);
		if digits.len() > 1 {
			b.push('.');
			b.push_str(&digits[1..]);
		}
		b.push('e');
		b.push_str(&exp.to_string());
	} else if exp < 0 {
		b.push_str("0.");
		b.push_str(&"0".repeat(usize::try_from(-exp - 1).unwrap_or(0)));
		b.push_str(digits);
	} else if len <= exp + 1 {
		b.push_str(digits);
		b.push_str(&"0".repeat(usize::try_from(exp + 1 - len).unwrap_or(0)));
		b.push_str(".0");
	} else {
		let at = usize::try_from(exp + 1).unwrap_or(0);
		b.push_str(&digits[..at]);
		b.push('.');
		b.push_str(&digits[at..]);
	}
	b
}

/// What `run` may touch outside the document.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
	/// Where `check` reads schema files and `write` writes. `None` turns
	/// both ops off.
	pub dir: Option<PathBuf>,
}

/// Parses `input` with `L`, applies a script of ops to it, and returns what
/// the ops printed. The script and its output are the same in every
/// wrapper language, which is what the cross-language tests compare. The
/// ops are listed in project/design.md under "Ops scripts".
pub fn run<L: Lib>(input: &str, script: &str, opt: &RunOptions) -> String {
	let mut r = Runner::<L> {
		input,
		opt,
		doc: L::parse(input),
		out: String::new(),
	};
	for line in script.split('\n') {
		let line = line.trim_end_matches('\r');
		if line.trim().is_empty() || line.starts_with('#') {
			continue;
		}
		r.say(&format!("> {line}"));
		let f: Vec<String> = line.split('\t').map(unesc).collect();
		r.op(&f);
	}
	r.out
}

struct Runner<'a, L: Lib> {
	input: &'a str,
	opt: &'a RunOptions,
	doc: L::Doc,
	out: String,
}

impl<L: Lib> Runner<'_, L> {
	fn say(&mut self, s: &str) {
		self.out.push_str(s);
		self.out.push('\n');
	}

	fn op(&mut self, f: &[String]) {
		let arg = |i: usize| f.get(i).map_or("", String::as_str);
		let rest = |i: usize| f.get(i..).unwrap_or(&[]).to_vec();
		match arg(0) {
			"parse" => self.parse(arg(1)),
			"text" => self.block(&self.doc.text()),
			"canonical" => self.block(&self.doc.canonical()),
			"diags" => self.diags(&self.doc.diags()),
			"errors" => self.say(&self.doc.errors().to_string()),
			"get" => {
				let s = self.get(arg(1), arg(2));
				self.say(&s);
			}
			"or" => self.or(arg(1), arg(2), &rest(3)),
			"has" => self.say(&self.doc.has(arg(1)).to_string()),
			"count" => self.say(&self.doc.count(arg(1)).to_string()),
			"line" => self.say(&self.doc.line(arg(1)).to_string()),
			"names" => self.say(&join_esc(&self.doc.names(arg(1)))),
			"each" => {
				for (i, n) in self.doc.each(arg(1)).iter().enumerate() {
					self.say(&format!("{i}\t{}\t{}", n.line, esc(&n.value)));
				}
			}
			"eachget" => {
				for (i, n) in self.doc.each(arg(2)).iter().enumerate() {
					let p = self.doc.child(&n.path, arg(3));
					let s = self.get(arg(1), &p);
					self.say(&format!("{i}\t{s}"));
				}
			}
			"put" => match parse_value(arg(1), &rest(3)) {
				Some(v) => {
					let r = self.doc.put(arg(2), v);
					self.result(r.err());
				}
				None => self.say("error badvalue"),
			},
			"syntax" => {
				let r = self.doc.put_syntax(arg(1), arg(2));
				self.result(r.err());
			}
			"comment" => {
				let r = self.doc.comment(arg(1), arg(2));
				self.result(r.err());
			}
			"remove" => {
				let n = self.doc.remove(arg(1));
				self.say(&n.to_string());
			}
			"check" => self.check(arg(1), arg(2)),
			"format" => self.say(&L::format_of(self.input).to_string()),
			"upgrade" => self.upgrade(arg(1)),
			"write" => self.write(arg(1)),
			_ => self.say("error unknownop"),
		}
	}

	fn parse(&mut self, level: &str) {
		let s = match level {
			"loose" => Strictness::Loose,
			"strict" => Strictness::Strict,
			_ => Strictness::Standard,
		};
		let (doc, err) = L::parse_keep(self.input, s);
		self.doc = doc;
		self.result(err);
	}

	fn result(&mut self, err: Option<Error>) {
		match err {
			None => self.say("ok"),
			Some(e) => self.say(&format!("error {}", e.kind)),
		}
	}

	fn diags(&mut self, ds: &[Diag]) {
		for d in ds {
			self.say(&format!("{}\t{}\t{}\t{}", d.line, d.severity, d.code, esc(&d.message)));
		}
	}

	/// Prints multi-line text one line per output line, so a golden file
	/// diff shows which line changed.
	fn block(&mut self, text: &str) {
		let mut lines: Vec<&str> = text.split('\n').collect();
		if lines.last() == Some(&"") {
			lines.pop();
		}
		for l in lines {
			self.say(&format!("| {}", esc(l)));
		}
		if !text.is_empty() && !text.ends_with('\n') {
			self.say("(no newline at end)");
		}
	}

	fn get(&self, typ: &str, path: &str) -> String {
		let d = &self.doc;
		let head = |s: Status, line: usize| format!("{s}\t{line}");
		match typ {
			"int" => {
				let g = d.get_int(path);
				format!("{}\t{}", head(g.status, g.line), g.value)
			}
			"float" => {
				let g = d.get_float(path);
				format!("{}\t{}", head(g.status, g.line), format_float(g.value))
			}
			"bool" => {
				let g = d.get_bool(path);
				format!("{}\t{}", head(g.status, g.line), g.value)
			}
			"str" => {
				let g = d.get_str(path);
				format!("{}\t{}", head(g.status, g.line), esc(&g.value))
			}
			"raw" => {
				let g = d.get_raw(path);
				format!("{}\t{}", head(g.status, g.line), esc(&g.value))
			}
			"strs" => {
				let g = d.get_strs(path);
				format!(
					"{}\t{}",
					head(g.status, g.line),
					counted(g.value.len(), &join_esc(&g.value))
				)
			}
			"ints" => {
				let g = d.get_ints(path);
				format!(
					"{}\t{}",
					head(g.status, g.line),
					counted(g.value.len(), &join_ints(&g.value))
				)
			}
			"floats" => {
				let g = d.get_floats(path);
				format!(
					"{}\t{}",
					head(g.status, g.line),
					counted(g.value.len(), &join_floats(&g.value))
				)
			}
			_ => "error badtype".to_owned(),
		}
	}

	fn or(&mut self, typ: &str, path: &str, fallback: &[String]) {
		let first = fallback.first().map_or("", String::as_str);
		let d = &self.doc;
		let s = match typ {
			"int" => first.parse::<i64>().ok().map(|n| d.int(path, n).to_string()),
			"float" => parse_float(first).map(|x| format_float(d.float(path, x))),
			"bool" => Some(d.bool(path, first == "true").to_string()),
			"str" => Some(esc(&d.str(path, first))),
			"raw" => Some(esc(&d.raw(path, first))),
			"strs" => {
				let fb: Vec<&str> = fallback.iter().map(String::as_str).collect();
				let v = d.strs(path, &fb);
				Some(counted(v.len(), &join_esc(&v)))
			}
			"ints" => match parse_value("ints", fallback) {
				Some(Value::Ints(fb)) => {
					let v = d.ints(path, &fb);
					Some(counted(v.len(), &join_ints(&v)))
				}
				_ => None,
			},
			"floats" => match parse_value("floats", fallback) {
				Some(Value::Floats(fb)) => {
					let v = d.floats(path, &fb);
					Some(counted(v.len(), &join_floats(&v)))
				}
				_ => None,
			},
			_ => {
				self.say("error badtype");
				return;
			}
		};
		self.say(&s.unwrap_or_else(|| "error badvalue".to_owned()));
	}

	/// Prints a schema check. Mode "all" adds the load's own diagnostics
	/// first, and "drop" does that and then drops what the schema declares.
	fn check(&mut self, name: &str, mode: &str) {
		let Some(path) = self.file(name) else { return };
		let text = match read_text(&path.to_string_lossy()) {
			Ok(t) => t,
			Err(e) => return self.result(Some(e)),
		};
		let schema = L::parse(&text);
		let mut diags = self.doc.check(&schema);
		if mode == "all" || mode == "drop" {
			let mut all = self.doc.diags();
			all.append(&mut diags);
			diags = all;
		}
		if mode == "drop" {
			diags = schema.drop_declared(diags);
		}
		self.diags(&diags);
	}

	fn upgrade(&mut self, from: &str) {
		let n = if from.is_empty() {
			Some(0)
		} else {
			from.parse::<u32>().ok()
		};
		let Some(n) = n else { return self.say("error badvalue") };
		match L::upgrade(self.input, n) {
			Ok(u) => {
				self.say(&format!("from {} changed {} unsure {}", u.from, u.changed, u.unsure));
				self.block(&u.text);
			}
			Err(e) => self.result(Some(e)),
		}
	}

	fn write(&mut self, name: &str) {
		let Some(path) = self.file(name) else { return };
		let path = path.to_string_lossy().into_owned();
		if let Err(e) = self.doc.write(&path) {
			return self.result(Some(e));
		}
		match read_text(&path) {
			Ok(t) => {
				self.say("ok");
				self.block(&t);
			}
			Err(e) => self.result(Some(e)),
		}
	}

	/// Keeps a script's file names inside `RunOptions::dir`.
	fn file(&mut self, name: &str) -> Option<PathBuf> {
		let plain = !name.is_empty() && name != ".." && Path::new(name).file_name().is_some_and(|b| b == name);
		match &self.opt.dir {
			Some(dir) if plain => Some(dir.join(name)),
			_ => {
				self.say("error unsupported");
				None
			}
		}
	}
}

fn parse_value(typ: &str, f: &[String]) -> Option<Value> {
	let first = f.first().map_or("", String::as_str);
	match typ {
		"int" => first.parse::<i64>().ok().map(Value::Int),
		"float" => parse_float(first).map(Value::Float),
		"bool" => match first {
			"true" => Some(Value::Bool(true)),
			"false" => Some(Value::Bool(false)),
			_ => None,
		},
		"str" => Some(Value::Str(first.to_owned())),
		"strs" => Some(Value::Strs(f.to_vec())),
		"ints" => f
			.iter()
			.map(|s| s.parse::<i64>().ok())
			.collect::<Option<Vec<_>>>()
			.map(Value::Ints),
		"floats" => f
			.iter()
			.map(|s| parse_float(s))
			.collect::<Option<Vec<_>>>()
			.map(Value::Floats),
		_ => None,
	}
}

/// Takes plain decimal and exponent forms only. Each language's own parser
/// takes a different set of spellings for infinity and NaN.
fn parse_float(s: &str) -> Option<f64> {
	let odd = |c: char| c.is_ascii_alphabetic() && c != 'e' && c != 'E' || c == '_';
	if s.is_empty() || s.contains(odd) {
		return None;
	}
	s.parse::<f64>().ok()
}

/// Undoes \t, \n, \r, and \\ in a script field.
fn unesc(s: &str) -> String {
	if !s.contains('\\') {
		return s.to_owned();
	}
	let mut b = String::with_capacity(s.len());
	let mut it = s.chars();
	while let Some(c) = it.next() {
		if c != '\\' {
			b.push(c);
			continue;
		}
		match it.next() {
			Some('t') => b.push('\t'),
			Some('n') => b.push('\n'),
			Some('r') => b.push('\r'),
			Some('\\') | None => b.push('\\'),
			Some(o) => {
				b.push('\\');
				b.push(o);
			}
		}
	}
	b
}

/// Makes a value safe for one tab-separated output field.
fn esc(s: &str) -> String {
	let mut b = String::with_capacity(s.len());
	for c in s.chars() {
		match c {
			'\\' => b.push_str("\\\\"),
			'\t' => b.push_str("\\t"),
			'\n' => b.push_str("\\n"),
			'\r' => b.push_str("\\r"),
			c if (c as u32) < 0x20 || c as u32 == 0x7f => {
				let _ = write!(b, "\\x{:02x}", c as u32); // writing to a String can't fail
			}
			c => b.push(c),
		}
	}
	b
}

/// Puts an array's length before its items, so `[]` and `[""]` print
/// differently.
fn counted(n: usize, items: &str) -> String {
	if n == 0 {
		"0".to_owned()
	} else {
		format!("{n}\t{items}")
	}
}

fn join_esc(v: &[String]) -> String {
	v.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\t")
}

fn join_ints(v: &[i64]) -> String {
	v.iter().map(ToString::to_string).collect::<Vec<_>>().join("\t")
}

fn join_floats(v: &[f64]) -> String {
	v.iter().map(|x| format_float(*x)).collect::<Vec<_>>().join("\t")
}

#[cfg(feature = "shcl1")]
pub mod v1;
#[cfg(feature = "shcl2")]
pub mod v2;
#[cfg(feature = "shcl3")]
pub mod v3;
