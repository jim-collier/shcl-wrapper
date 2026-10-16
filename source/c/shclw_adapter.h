// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// What the version adapters and run.c share, from shclw.c. Not for programs.

#ifndef SHCLW_ADAPTER_H
#define SHCLW_ADAPTER_H

#include "shclw.h"

#ifdef __cplusplus
extern "C" {
#endif

// Fills *err, when there is one, and returns kind. path and why may be NULL.
shclw_kind shclw_i_fail(shclw_error *err, shclw_kind kind, const char *path, const char *why, int sys_errno);
shclw_kind shclw_i_ok(shclw_error *err);

// malloc of at least 1 byte, so NULL always means out of memory.
void *shclw_i_alloc(size_t n);
// A copy of p[0..n) with a NUL after it. p may be NULL when n is 0.
char *shclw_i_dup(const char *p, size_t n, size_t *len);
// Copies of the items in one block, each with a NUL after it.
shclw_string *shclw_i_copy_strs(const shclw_string *items, size_t n, size_t *count);
int64_t *shclw_i_copy_ints(const int64_t *items, size_t n, size_t *count);
double *shclw_i_copy_floats(const double *items, size_t n, size_t *count);

// The length of the UTF-8 sequence at s, and its code point in *r, or 0 when
// it isn't a valid one. Go's rules: no overlongs, surrogates, or past U+10FFFF.
size_t shclw_i_utf8_next(const char *s, size_t n, uint32_t *r);
bool shclw_i_utf8_valid(const char *s, size_t n);

// Infinity and NaN read as SHCLW_BAD_TYPE on every version. An array keeps its
// length, with 0 for each item that wasn't a number.
void shclw_i_finite(shclw_got_float *g);
void shclw_i_finite_all(shclw_got_floats *g);

// A put's value, Go's shclw.Value. n is the length of str or of the array.
typedef enum {
	SHCLW_I_INT,
	SHCLW_I_FLOAT,
	SHCLW_I_BOOL,
	SHCLW_I_STR,
	SHCLW_I_STRS,
	SHCLW_I_INTS,
	SHCLW_I_FLOATS
} shclw_i_kind;
typedef struct {
	shclw_i_kind kind;
	int64_t i;
	double f;
	bool b;
	const char *str;
	const shclw_string *strs;
	const int64_t *ints;
	const double *floats;
	size_t n;
} shclw_i_value;

// A value of kind with every other field 0, and n set.
shclw_i_value shclw_i_value_of(shclw_i_kind kind, size_t n);
// Go's VerifyPut: reads back what a setter wrote, since shcl can take a value
// and read back something else.
shclw_kind shclw_i_verify_put(shclw_doc *d, const char *path, const shclw_i_value *v, shclw_error *err);
// The adapter's half of a put: the set alone, with no read-back. shclw.c
// checks the value first and reads it back after.
shclw_kind shclw_i_set(shclw_doc *d, const char *path, const shclw_i_value *v, shclw_error *err);
// Go's VerifySyntax. alone has the text set at "k", and line is "k: text"
// parsed.
shclw_kind shclw_i_verify_syntax(shclw_doc *d, const char *path, shclw_doc *alone, shclw_doc *line, shclw_error *err);

// The first of each name, compared without ASCII case, as shcl matches names.
shclw_string *shclw_i_dedupe_names(const shclw_string *names, size_t n, size_t *count);
// Copies of the nodes in one block, each string with a NUL after it.
shclw_node *shclw_i_copy_nodes(const shclw_node *nodes, size_t n, size_t *count);

// The shortest digits that read back as f, and the closest of those: Go's
// strconv.FormatFloat(f, 'e', -1, 64), or with single, as a float32. digits
// gets them with no point, at least 40 bytes, and *exp the power of 10 of the
// first. f is finite.
void shclw_i_shortest(double f, bool single, char *digits, int *exp);

#ifdef __cplusplus
}
#endif

#endif
