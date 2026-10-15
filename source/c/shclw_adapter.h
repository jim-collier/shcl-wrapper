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

#ifdef __cplusplus
}
#endif

#endif
