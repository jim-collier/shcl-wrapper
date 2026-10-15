// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};

/// A test's own folder under the temp root, removed on drop. Drop still
/// runs when a failed assert unwinds, so a failing run cleans up too.
#[derive(Debug)]
pub struct Scratch(PathBuf);

impl Scratch {
	pub fn new(name: &str) -> Self {
		let base = std::env::var_os("TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
		let dir = base.join(format!("shclw-rust-{name}-{}", std::process::id()));
		let _ = fs::remove_dir_all(&dir); // a dead run's, if the pid came around again
		fs::create_dir_all(&dir).unwrap();
		Scratch(dir)
	}

	pub fn path(&self) -> &Path {
		&self.0
	}
}

impl Drop for Scratch {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.0); // only ever the folder new() made
	}
}
