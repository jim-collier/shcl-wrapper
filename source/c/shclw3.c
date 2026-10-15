// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// The wrapper over shcl 3.0.
//
// Documents keep their lines as typed, so shclw_text and shclw_write give back
// what was read plus the edits, where 3.0 can manage that, and the canonical
// form where it can't. shclw_write refuses with SHCLW_ERR_REFUSED rather than
// drop a line.
//
// Every read copies its answer out and then gives back shcl's read memory, so
// a program that reads in a loop doesn't grow (2026101513330046).

#include "shclw_adapter.h"

#if SHCLW_SHCL != 3
#error "shclw3.c is the adapter for shcl 3.0, and this shcl.h is another version"
#endif

#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct shclw_doc {
	shcl_doc *d;
};

static shclw_status status_of(shcl_status s) {
	switch (s) {
	case SHCL_GOOD: return SHCLW_GOOD;
	case SHCL_EMPTY: return SHCLW_EMPTY;
	case SHCL_NOT_FOUND: return SHCLW_NOT_FOUND;
	case SHCL_BAD_TYPE: return SHCLW_BAD_TYPE;
	case SHCL_MULTIPLE: return SHCLW_MULTIPLE;
	case SHCL_BAD_PATH: return SHCLW_BAD_PATH;
	}
	return SHCLW_BAD_PATH;
}

static shcl_strictness level_of(shclw_strictness s) {
	switch (s) {
	case SHCLW_LOOSE: return SHCL_LOOSE;
	case SHCLW_STRICT: return SHCL_STRICT;
	case SHCLW_STANDARD: return SHCL_STANDARD;
	}
	return SHCL_STANDARD;
}

// The format a file's Format line names, or 0.
static int64_t format_of(const char *text, size_t len) {
	shcl_read_i64 r = shcl_read_format_version(text, len);
	return r.status == SHCL_GOOD ? r.value : 0;
}

static shclw_kind too_new(shclw_error *err, const char *path, int64_t format) {
	char why[128];
	(void)snprintf(why, sizeof why, "format %" PRId64 ", and shcl %s reads up to %d", format, SHCLW_SHCL_VERSION,
	               SHCLW_FORMAT_MAJOR);
	return shclw_i_fail(err, SHCLW_ERR_TOO_NEW, path, why, 0);
}

shclw_doc *shclw_new(void) { return shclw_parse("", 0); }

shclw_doc *shclw_parse(const char *text, size_t len) {
	shclw_doc *d = NULL;
	shclw_parse_with(text, len, SHCLW_STANDARD, &d, NULL);
	return d;
}

shclw_kind shclw_parse_with(const char *text, size_t len, shclw_strictness level, shclw_doc **doc, shclw_error *err) {
	*doc = NULL;
	shclw_doc *d = (shclw_doc *)malloc(sizeof *d);
	if (!d) {
		return shclw_i_fail(err, SHCLW_ERR_NO_MEMORY, NULL, NULL, 0);
	}
	d->d = shcl_parse_keep_lines(text ? text : "", text ? len : 0, level_of(level));
	if (!d->d) {
		free(d);
		return shclw_i_fail(err, SHCLW_ERR_NO_MEMORY, NULL, NULL, 0);
	}
	*doc = d;
	if (shcl_strict_failed(d->d)) {
		char why[64];
		(void)snprintf(why, sizeof why, "the load has %zu errors", shcl_error_count(d->d));
		return shclw_i_fail(err, SHCLW_ERR_STRICT, NULL, why, 0);
	}
	return shclw_i_ok(err);
}

shclw_kind shclw_open(const char *path, shclw_doc **doc, shclw_error *err) {
	return shclw_open_with(path, SHCLW_STANDARD, doc, err);
}

shclw_kind shclw_open_with(const char *path, shclw_strictness level, shclw_doc **doc, shclw_error *err) {
	*doc = NULL;
	char *text = NULL;
	size_t len = 0;
	shclw_kind kind = shclw_read_text(path, &text, &len, err);
	if (kind == SHCLW_OK) {
		int64_t format = format_of(text, len);
		if (format > SHCLW_FORMAT_MAJOR) {
			kind = too_new(err, path, format);
		}
	}
	if (kind != SHCLW_OK) {
		free(text);
		*doc = shclw_new();
		return *doc ? kind : shclw_i_fail(err, SHCLW_ERR_NO_MEMORY, path, NULL, 0);
	}
	kind = shclw_parse_with(text, len, level, doc, NULL);
	free(text);
	switch (kind) {
	case SHCLW_OK: return shclw_i_ok(err);
	case SHCLW_ERR_STRICT: {
		char why[64];
		(void)snprintf(why, sizeof why, "the load has %zu errors", *doc ? shcl_error_count((*doc)->d) : 0);
		return shclw_i_fail(err, kind, path, why, 0);
	}
	default: return shclw_i_fail(err, kind, path, NULL, 0);
	}
}

void shclw_free(shclw_doc *d) {
	if (d) {
		shcl_free(d->d);
		free(d);
	}
}

shcl_doc *shclw_inner(shclw_doc *d) { return d->d; }

char *shclw_text(shclw_doc *d, size_t *len) {
	// Without the lines kept, this is the canonical form.
	shcl_str s = shcl_to_text_keep_lines(d->d, NULL);
	char *out = shclw_i_dup(s.p, s.n, len);
	shcl_reads_release(d->d);
	return out;
}

char *shclw_canonical(shclw_doc *d, size_t *len) {
	shcl_str s = shcl_to_canonical(d->d);
	char *out = shclw_i_dup(s.p, s.n, len);
	shcl_reads_release(d->d);
	return out;
}

shclw_kind shclw_write(shclw_doc *d, const char *path, shclw_error *err) {
	int kept = 0;
	shcl_str s = shcl_to_text_keep_lines(d->d, &kept);
	size_t lost = shcl_lost_count(d->d);
	if (!kept && lost > 0) {
		shcl_reads_release(d->d);
		char why[64];
		(void)snprintf(why, sizeof why, "%zu lines would be lost", lost);
		return shclw_i_fail(err, SHCLW_ERR_REFUSED, path, why, 0);
	}
	shcl_write_status why = SHCL_WRITE_OK;
	errno = 0;
	int wrote = shcl_write_file_atomic(path, s.p, s.n, &why);
	int e = errno;
	shcl_reads_release(d->d);
	if (wrote) {
		return shclw_i_ok(err);
	}
	shclw_kind kind = why == SHCL_WRITE_NOT_FOUND ? SHCLW_ERR_NOT_FOUND : SHCLW_ERR_IO;
	return shclw_i_fail(err, kind, path, shcl_write_status_name(why), e);
}

shclw_diag *shclw_diags(shclw_doc *d, size_t *n) {
	*n = 0;
	size_t count = shcl_diag_count(d->d);
	if (count > (SIZE_MAX / 2) / sizeof(shclw_diag)) {
		return NULL;
	}
	size_t bytes = count * sizeof(shclw_diag);
	for (size_t i = 0; i < count; i++) {
		size_t more = strlen(shcl_diag_code(d->d, i)) + shcl_diag_message(d->d, i).n + 2;
		if (more >= SIZE_MAX / 2 - bytes) {
			return NULL;
		}
		bytes += more;
	}
	shclw_diag *out = (shclw_diag *)shclw_i_alloc(bytes);
	if (!out) {
		return NULL;
	}
	char *text = (char *)(out + count);
	for (size_t i = 0; i < count; i++) {
		const char *code = shcl_diag_code(d->d, i);
		size_t code_len = strlen(code);
		shcl_str msg = shcl_diag_message(d->d, i);
		out[i].line = shcl_diag_line(d->d, i);
		out[i].severity = shcl_diag_severity(d->d, i) == SHCL_SEV_ERROR ? SHCLW_SEV_ERROR : SHCLW_SEV_HINT;
		memcpy(text, code, code_len + 1);
		out[i].code.p = text;
		out[i].code.n = code_len;
		text += code_len + 1;
		if (msg.n) {
			memcpy(text, msg.p, msg.n);
		}
		text[msg.n] = '\0';
		out[i].message.p = text;
		out[i].message.n = msg.n;
		text += msg.n + 1;
	}
	*n = count;
	return out;
}

size_t shclw_errors(shclw_doc *d) { return shcl_error_count(d->d); }

int64_t shclw_int(shclw_doc *d, const char *path, int64_t fallback) {
	int64_t v = shcl_get_int_or(d->d, path, strlen(path), fallback);
	shcl_reads_release(d->d);
	return v;
}

// Through shclw_get_float, so infinity gets the fallback too.
double shclw_float(shclw_doc *d, const char *path, double fallback) {
	shclw_got_float g = shclw_get_float(d, path);
	return g.status == SHCLW_GOOD ? g.value : fallback;
}

bool shclw_bool(shclw_doc *d, const char *path, bool fallback) {
	int v = shcl_get_bool_or(d->d, path, strlen(path), fallback ? 1 : 0);
	shcl_reads_release(d->d);
	return v != 0;
}

char *shclw_str(shclw_doc *d, const char *path, const char *fallback, size_t fallback_len, size_t *len) {
	shclw_got_str g = shclw_get_str(d, path);
	if (g.status == SHCLW_GOOD) {
		if (len) {
			*len = g.len;
		}
		return g.value;
	}
	free(g.value);
	return shclw_i_dup(fallback, fallback ? fallback_len : 0, len);
}

char *shclw_raw(shclw_doc *d, const char *path, const char *fallback, size_t fallback_len, size_t *len) {
	shclw_got_str g = shclw_get_raw(d, path);
	if (g.status == SHCLW_GOOD) {
		if (len) {
			*len = g.len;
		}
		return g.value;
	}
	free(g.value);
	return shclw_i_dup(fallback, fallback ? fallback_len : 0, len);
}

shclw_string *shclw_strs(shclw_doc *d, const char *path, const shclw_string *fallback, size_t fallback_n, size_t *n) {
	shclw_got_strs g = shclw_get_strs(d, path);
	if (g.status == SHCLW_GOOD) {
		*n = g.n;
		return g.value;
	}
	free(g.value);
	return shclw_i_copy_strs(fallback, fallback_n, n);
}

int64_t *shclw_ints(shclw_doc *d, const char *path, const int64_t *fallback, size_t fallback_n, size_t *n) {
	shclw_got_ints g = shclw_get_ints(d, path);
	if (g.status == SHCLW_GOOD) {
		*n = g.n;
		return g.value;
	}
	free(g.value);
	return shclw_i_copy_ints(fallback, fallback_n, n);
}

double *shclw_floats(shclw_doc *d, const char *path, const double *fallback, size_t fallback_n, size_t *n) {
	shclw_got_floats g = shclw_get_floats(d, path);
	if (g.status == SHCLW_GOOD) {
		*n = g.n;
		return g.value;
	}
	free(g.value);
	return shclw_i_copy_floats(fallback, fallback_n, n);
}

// shcl's own reads leave the line out of their result. shcl_line gives it for
// a path that names one node, which is when Go's and Rust's reads have one.
static size_t line_of(shclw_doc *d, const char *path, size_t plen) { return shcl_line(d->d, path, plen); }

shclw_got_int shclw_get_int(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_i64 r = shcl_read_int(d->d, path, plen);
	shclw_got_int g = {r.value, status_of(r.status), line_of(d, path, plen)};
	shcl_reads_release(d->d);
	return g;
}

shclw_got_float shclw_get_float(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_f64 r = shcl_read_float(d->d, path, plen);
	shclw_got_float g = {r.value, status_of(r.status), line_of(d, path, plen)};
	shcl_reads_release(d->d);
	shclw_i_finite(&g);
	return g;
}

shclw_got_bool shclw_get_bool(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_bool r = shcl_read_bool_(d->d, path, plen);
	shclw_got_bool g = {r.value != 0, status_of(r.status), line_of(d, path, plen)};
	shcl_reads_release(d->d);
	return g;
}

static shclw_got_str got_str(shclw_doc *d, const char *path, size_t plen, shcl_read_str r) {
	shclw_got_str g = {NULL, 0, status_of(r.status), line_of(d, path, plen)};
	g.value = shclw_i_dup(r.value.p, r.value.n, &g.len);
	shcl_reads_release(d->d);
	return g;
}

shclw_got_str shclw_get_str(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	return got_str(d, path, plen, shcl_read_string(d->d, path, plen));
}

shclw_got_str shclw_get_raw(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	return got_str(d, path, plen, shcl_read_raw(d->d, path, plen));
}

shclw_got_strs shclw_get_strs(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_str_arr r = shcl_read_string_array(d->d, path, plen);
	shclw_got_strs g = {NULL, 0, status_of(r.status), line_of(d, path, plen)};
	size_t n = r.values ? r.n : 0;
	shclw_string *items = (shclw_string *)shclw_i_alloc(n * sizeof(shclw_string));
	if (items) {
		for (size_t i = 0; i < n; i++) {
			items[i].p = r.values[i].p;
			items[i].n = r.values[i].n;
		}
		g.value = shclw_i_copy_strs(items, n, &g.n);
		free(items);
	}
	shcl_reads_release(d->d);
	return g;
}

shclw_got_ints shclw_get_ints(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_i64_arr r = shcl_read_int_array(d->d, path, plen);
	shclw_got_ints g = {NULL, 0, status_of(r.status), line_of(d, path, plen)};
	g.value = shclw_i_copy_ints(r.values, r.values ? r.n : 0, &g.n);
	shcl_reads_release(d->d);
	return g;
}

shclw_got_floats shclw_get_floats(shclw_doc *d, const char *path) {
	size_t plen = strlen(path);
	shcl_read_f64_arr r = shcl_read_float_array(d->d, path, plen);
	shclw_got_floats g = {NULL, 0, status_of(r.status), line_of(d, path, plen)};
	g.value = shclw_i_copy_floats(r.values, r.values ? r.n : 0, &g.n);
	shcl_reads_release(d->d);
	shclw_i_finite_all(&g);
	return g;
}
