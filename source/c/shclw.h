// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// shclw: a small, stable wrapper over the shcl version a program builds
// against. The calls are the same as the Go and Rust wrappers', in snake case.
// See project/design.md.
//
// Build: shclw.c, the adapter that matches the shcl.h on the include path
// (shclw3.c for 3.0), and run.c for shclw_run. shcl.h's own
// SHCL_IMPLEMENTATION goes in one .c of the program's, as shcl asks.
//
// Include this before any system header, since shcl.h has to be first.
//
// Memory: the strings, arrays, text, and diagnostics a shclw call hands back
// are malloc'd, each in one block, and the caller frees them with free().
// Documents are freed with shclw_free. Reads copy their value out and give
// shcl's read memory back before they return.
//
// Strings: anything shclw hands back has its length and a NUL after it. Go by
// the length, since a value can hold a NUL. Paths and file names are plain
// NUL-terminated strings.
//
// Errors: a call that can fail returns a shclw_kind, SHCLW_OK on success, and
// fills *err when err isn't NULL.
//
// A document is for one thread at a time, reads included, as in shcl.

#ifndef SHCLW_H
#define SHCLW_H

#include "shcl.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// shcl.h has no version macro before 3.0. SHCL_FORMAT_MAJOR is new in 3.0.
#if defined(SHCL_FORMAT_MAJOR) && SHCL_FORMAT_MAJOR == 3
#define SHCLW_SHCL 3
#define SHCLW_FORMAT_MAJOR 3
// The shcl the wrapper is tested against. A shcl.h can't say its own.
#define SHCLW_SHCL_VERSION "3.0.0-dev+ea0998fa"
#elif defined(SHCL_FORMAT_MAJOR)
#error "shclw: this shcl.h is newer than 3.0, which shclw doesn't wrap yet"
#else
#error "shclw: this shcl.h is 1.2 or 2.0, which shclw doesn't wrap in C yet"
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct shclw_doc shclw_doc;

typedef enum { SHCLW_GOOD, SHCLW_EMPTY, SHCLW_NOT_FOUND, SHCLW_BAD_TYPE, SHCLW_MULTIPLE, SHCLW_BAD_PATH } shclw_status;

typedef enum { SHCLW_LOOSE, SHCLW_STANDARD, SHCLW_STRICT } shclw_strictness;

typedef enum { SHCLW_SEV_ERROR, SHCLW_SEV_HINT } shclw_severity;

// The same kinds as Go and Rust, in the same order. SHCLW_ERR_NO_MEMORY is
// C's own.
typedef enum {
	SHCLW_OK,
	SHCLW_ERR_NOT_FOUND,
	SHCLW_ERR_IO,
	SHCLW_ERR_STRICT,
	SHCLW_ERR_REFUSED,
	SHCLW_ERR_BAD_PATH,
	SHCLW_ERR_BAD_VALUE,
	SHCLW_ERR_TOO_NEW,
	SHCLW_ERR_UNSUPPORTED,
	SHCLW_ERR_AMBIGUOUS,
	SHCLW_ERR_BACKUP_TAKEN,
	SHCLW_ERR_NO_READ_BACK,
	SHCLW_ERR_NO_MEMORY
} shclw_kind;

// msg reads like Go's Error(): kind, then the path and why. sys_errno is the
// errno behind an SHCLW_ERR_IO, or 0.
typedef struct {
	shclw_kind kind;
	int sys_errno;
	char msg[512];
} shclw_error;

// A string with its length. p has a NUL at p[n], and may hold others before it.
typedef struct {
	const char *p;
	size_t n;
} shclw_string;

typedef struct {
	size_t line;
	shclw_severity severity;
	shclw_string code;
	shclw_string message;
} shclw_diag;

// A read with its status. Check status before trusting value: an array that
// reads as SHCLW_BAD_TYPE still has its items, with 0 for the bad ones. line
// is 0 unless the path names one node. A value pointer is the caller's to
// free(), and is NULL only when memory ran out.
typedef struct {
	int64_t value;
	shclw_status status;
	size_t line;
} shclw_got_int;

typedef struct {
	double value;
	shclw_status status;
	size_t line;
} shclw_got_float;

typedef struct {
	bool value;
	shclw_status status;
	size_t line;
} shclw_got_bool;

typedef struct {
	char *value;
	size_t len;
	shclw_status status;
	size_t line;
} shclw_got_str;

// The items' text is in the same block as the array, so one free(value) frees
// all of it.
typedef struct {
	shclw_string *value;
	size_t n;
	shclw_status status;
	size_t line;
} shclw_got_strs;

typedef struct {
	int64_t *value;
	size_t n;
	shclw_status status;
	size_t line;
} shclw_got_ints;

typedef struct {
	double *value;
	size_t n;
	shclw_status status;
	size_t line;
} shclw_got_floats;

// "Good", "NotFound", and so on, as the other wrappers print them.
const char *shclw_status_name(shclw_status s);
// "notfound", "io", and so on.
const char *shclw_kind_name(shclw_kind k);
// "error" or "hint".
const char *shclw_severity_name(shclw_severity s);

// An empty document. NULL only when memory ran out.
shclw_doc *shclw_new(void);
// Never fails on the text's account; problems are in shclw_diags. NULL only
// when memory ran out. text needn't end in a NUL.
shclw_doc *shclw_parse(const char *text, size_t len);
// Fails only at SHCLW_STRICT, with SHCLW_ERR_STRICT. *doc is set either way.
shclw_kind shclw_parse_with(const char *text, size_t len, shclw_strictness level, shclw_doc **doc, shclw_error *err);
// Reads and parses a file. A missing file is SHCLW_ERR_NOT_FOUND, and one
// written for a newer format SHCLW_ERR_TOO_NEW. On a failure *doc is an empty
// document, so reads give their fallbacks. Free it either way.
shclw_kind shclw_open(const char *path, shclw_doc **doc, shclw_error *err);
shclw_kind shclw_open_with(const char *path, shclw_strictness level, shclw_doc **doc, shclw_error *err);
void shclw_free(shclw_doc *d);

// The document as it would be saved. Caller frees.
char *shclw_text(shclw_doc *d, size_t *len);
// The document in canonical form. Caller frees.
char *shclw_canonical(shclw_doc *d, size_t *len);
// Saves shclw_text atomically. SHCLW_ERR_REFUSED when the save would lose
// lines the load couldn't use.
shclw_kind shclw_write(shclw_doc *d, const char *path, shclw_error *err);

// The load's diagnostics. Caller frees the block. NULL only when memory ran out.
shclw_diag *shclw_diags(shclw_doc *d, size_t *n);
// How many of them are errors.
size_t shclw_errors(shclw_doc *d);

// The value, or the fallback when there's no good one.
int64_t shclw_int(shclw_doc *d, const char *path, int64_t fallback);
double shclw_float(shclw_doc *d, const char *path, double fallback);
bool shclw_bool(shclw_doc *d, const char *path, bool fallback);
// A copy of the value or the fallback, which may be NULL for "". Caller frees.
char *shclw_str(shclw_doc *d, const char *path, const char *fallback, size_t fallback_len, size_t *len);
char *shclw_raw(shclw_doc *d, const char *path, const char *fallback, size_t fallback_len, size_t *len);
// Arrays copy the same way, as one block. Caller frees.
shclw_string *shclw_strs(shclw_doc *d, const char *path, const shclw_string *fallback, size_t fallback_n, size_t *n);
int64_t *shclw_ints(shclw_doc *d, const char *path, const int64_t *fallback, size_t fallback_n, size_t *n);
double *shclw_floats(shclw_doc *d, const char *path, const double *fallback, size_t fallback_n, size_t *n);

// The value with its status and line. A float that isn't finite is
// SHCLW_BAD_TYPE, as on every version.
shclw_got_int shclw_get_int(shclw_doc *d, const char *path);
shclw_got_float shclw_get_float(shclw_doc *d, const char *path);
shclw_got_bool shclw_get_bool(shclw_doc *d, const char *path);
shclw_got_str shclw_get_str(shclw_doc *d, const char *path);
shclw_got_str shclw_get_raw(shclw_doc *d, const char *path);
shclw_got_strs shclw_get_strs(shclw_doc *d, const char *path);
shclw_got_ints shclw_get_ints(shclw_doc *d, const char *path);
shclw_got_floats shclw_get_floats(shclw_doc *d, const char *path);

// The shcl document underneath. A shclw read gives shcl's read memory back,
// so a result from shcl's own read calls on it is good only until the next
// shclw call.
shcl_doc *shclw_inner(shclw_doc *d);

// A whole file, which has to be UTF-8. A missing one is SHCLW_ERR_NOT_FOUND.
// Caller frees *text.
shclw_kind shclw_read_text(const char *path, char **text, size_t *len, shclw_error *err);

// Runs an ops script on a document parsed from input, and gives back what the
// ops printed (project/design.md, "Ops scripts"). dir is where `check` and
// `write` may touch files; NULL turns them off. Caller frees. NULL only when
// memory ran out.
char *shclw_run(const char *input, size_t len, const char *script, size_t script_len, const char *dir, size_t *out_len);

#ifdef __cplusplus
}
#endif

#endif
