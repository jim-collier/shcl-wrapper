# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
## vNEXT - DATE

### Notes

### Added

### Changed

### Removed

### Other work
-->

## Unreleased

### Added

- Go and Rust wrappers over shcl 1.2, 2.0, and 3.0, with one API across all of them.

- Ops scripts, a text form of reads and edits, through `shclw.Run` and `shclw::run`.

- A pipeline with shared test cases for both languages, and fuzzing of Go against Rust.

### Changed

- The project is now named shclwrap, and the Go module path is `github.com/jim-collier/shclwrap/source/go`.
