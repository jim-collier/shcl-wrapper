// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// What every version adapter shares.

#include "shclw_adapter.h"

#include <errno.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const char *shclw_status_name(shclw_status s) {
	switch (s) {
	case SHCLW_GOOD: return "Good";
	case SHCLW_EMPTY: return "Empty";
	case SHCLW_NOT_FOUND: return "NotFound";
	case SHCLW_BAD_TYPE: return "BadType";
	case SHCLW_MULTIPLE: return "Multiple";
	case SHCLW_BAD_PATH: return "BadPath";
	}
	return "Status(?)";
}

const char *shclw_kind_name(shclw_kind k) {
	switch (k) {
	case SHCLW_OK: return "ok";
	case SHCLW_ERR_NOT_FOUND: return "notfound";
	case SHCLW_ERR_IO: return "io";
	case SHCLW_ERR_STRICT: return "strict";
	case SHCLW_ERR_REFUSED: return "refused";
	case SHCLW_ERR_BAD_PATH: return "badpath";
	case SHCLW_ERR_BAD_VALUE: return "badvalue";
	case SHCLW_ERR_TOO_NEW: return "toonew";
	case SHCLW_ERR_UNSUPPORTED: return "unsupported";
	case SHCLW_ERR_AMBIGUOUS: return "ambiguous";
	case SHCLW_ERR_BACKUP_TAKEN: return "backuptaken";
	case SHCLW_ERR_NO_READ_BACK: return "noreadback";
	case SHCLW_ERR_NO_MEMORY: return "nomemory";
	}
	return "unknown";
}

const char *shclw_severity_name(shclw_severity s) { return s == SHCLW_SEV_ERROR ? "error" : "hint"; }

shclw_kind shclw_i_fail(shclw_error *err, shclw_kind kind, const char *path, const char *why, int sys_errno) {
	if (!err) {
		return kind;
	}
	err->kind = kind;
	err->sys_errno = sys_errno;
	// A message cut short by the buffer is still a message, so snprintf's
	// truncation isn't an error here.
	int at = snprintf(err->msg, sizeof err->msg, "shclw: %s", shclw_kind_name(kind));
	if (at > 0 && path && (size_t)at < sizeof err->msg) {
		at += snprintf(err->msg + at, sizeof err->msg - (size_t)at, " \"%s\"", path);
	}
	if (at > 0 && why && (size_t)at < sizeof err->msg) {
		at += snprintf(err->msg + at, sizeof err->msg - (size_t)at, ": %s", why);
	}
	if (at > 0 && sys_errno && (size_t)at < sizeof err->msg) {
		(void)snprintf(err->msg + at, sizeof err->msg - (size_t)at, ": %s", strerror(sys_errno));
	}
	return kind;
}

shclw_kind shclw_i_ok(shclw_error *err) {
	if (err) {
		err->kind = SHCLW_OK;
		err->sys_errno = 0;
		err->msg[0] = '\0';
	}
	return SHCLW_OK;
}

void *shclw_i_alloc(size_t n) { return malloc(n ? n : 1); }

char *shclw_i_dup(const char *p, size_t n, size_t *len) {
	if (len) {
		*len = 0;
	}
	if (n == SIZE_MAX) {
		return NULL;
	}
	char *out = (char *)malloc(n + 1);
	if (!out) {
		return NULL;
	}
	if (n) {
		memcpy(out, p, n);
	}
	out[n] = '\0';
	if (len) {
		*len = n;
	}
	return out;
}

shclw_string *shclw_i_copy_strs(const shclw_string *items, size_t n, size_t *count) {
	*count = 0;
	if (n > (SIZE_MAX / 2) / sizeof(shclw_string)) {
		return NULL;
	}
	size_t text_bytes = 0;
	for (size_t i = 0; i < n; i++) {
		if (items[i].n >= SIZE_MAX / 4 - text_bytes) {
			return NULL;
		}
		text_bytes += items[i].n + 1;
	}
	shclw_string *out = (shclw_string *)shclw_i_alloc(n * sizeof(shclw_string) + text_bytes);
	if (!out) {
		return NULL;
	}
	char *text = (char *)(out + n);
	for (size_t i = 0; i < n; i++) {
		size_t len = items[i].n;
		if (len) {
			memcpy(text, items[i].p, len);
		}
		// The analyzer loses the sum above and takes the 1-byte allocation.
		text[len] = '\0'; // NOLINT(clang-analyzer-security.ArrayBound)
		out[i].p = text;
		out[i].n = len;
		text += len + 1;
	}
	*count = n;
	return out;
}

int64_t *shclw_i_copy_ints(const int64_t *items, size_t n, size_t *count) {
	*count = 0;
	if (n > SIZE_MAX / sizeof(int64_t)) {
		return NULL;
	}
	int64_t *out = (int64_t *)shclw_i_alloc(n * sizeof(int64_t));
	if (!out) {
		return NULL;
	}
	if (n) {
		memcpy(out, items, n * sizeof(int64_t));
	}
	*count = n;
	return out;
}

double *shclw_i_copy_floats(const double *items, size_t n, size_t *count) {
	*count = 0;
	if (n > SIZE_MAX / sizeof(double)) {
		return NULL;
	}
	double *out = (double *)shclw_i_alloc(n * sizeof(double));
	if (!out) {
		return NULL;
	}
	if (n) {
		memcpy(out, items, n * sizeof(double));
	}
	*count = n;
	return out;
}

size_t shclw_i_utf8_next(const char *s, size_t n, uint32_t *r) {
	const unsigned char *u = (const unsigned char *)s;
	if (n == 0) {
		return 0;
	}
	if (u[0] < 0x80) {
		*r = u[0];
		return 1;
	}
	// clang-format off
	static const struct { unsigned char lo, hi, mask; size_t size; uint32_t least; } leads[] = {
		{0xC2, 0xDF, 0x1F, 2, 0x80},
		{0xE0, 0xEF, 0x0F, 3, 0x800},
		{0xF0, 0xF4, 0x07, 4, 0x10000},
	};
	// clang-format on
	size_t size = 0;
	uint32_t c = 0;
	uint32_t least = 0;
	for (size_t i = 0; i < sizeof leads / sizeof leads[0]; i++) {
		if (u[0] >= leads[i].lo && u[0] <= leads[i].hi) {
			size = leads[i].size;
			c = u[0] & (uint32_t)leads[i].mask;
			least = leads[i].least;
		}
	}
	if (!size) {
		return 0;
	}
	if (n < size) {
		return 0;
	}
	for (size_t i = 1; i < size; i++) {
		if ((u[i] & 0xC0u) != 0x80u) {
			return 0;
		}
		c = (c << 6) | (u[i] & 0x3Fu);
	}
	if (c < least || c > 0x10FFFF || (c >= 0xD800 && c <= 0xDFFF)) {
		return 0;
	}
	*r = c;
	return size;
}

bool shclw_i_utf8_valid(const char *s, size_t n) {
	uint32_t r = 0;
	for (size_t i = 0; i < n;) {
		size_t size = shclw_i_utf8_next(s + i, n - i, &r);
		if (!size) {
			return false;
		}
		i += size;
	}
	return true;
}

void shclw_i_finite(shclw_got_float *g) {
	if (g->status == SHCLW_GOOD && !isfinite(g->value)) {
		g->status = SHCLW_BAD_TYPE;
		g->value = 0;
	}
}

void shclw_i_finite_all(shclw_got_floats *g) {
	if (g->status != SHCLW_GOOD || !g->value) {
		return;
	}
	for (size_t i = 0; i < g->n; i++) {
		if (!isfinite(g->value[i])) {
			g->value[i] = 0;
			g->status = SHCLW_BAD_TYPE;
		}
	}
}

shclw_kind shclw_read_text(const char *path, char **text, size_t *len, shclw_error *err) {
	*text = NULL;
	*len = 0;
	FILE *f = fopen(path, "rb");
	if (!f) {
		int e = errno;
		return shclw_i_fail(err, e == ENOENT ? SHCLW_ERR_NOT_FOUND : SHCLW_ERR_IO, path, NULL, e);
	}
	size_t cap = 4096;
	size_t n = 0;
	char *buf = (char *)malloc(cap);
	shclw_kind kind = SHCLW_OK;
	int e = 0;
	while (buf) {
		if (cap - n < 2) {
			char *grown = cap <= SIZE_MAX / 2 ? (char *)realloc(buf, cap * 2) : NULL;
			if (!grown) {
				free(buf);
				buf = NULL;
				break;
			}
			buf = grown;
			cap *= 2;
		}
		size_t want = cap - n - 1;
		size_t got = fread(buf + n, 1, want, f);
		n += got;
		if (got < want) {
			if (ferror(f)) {
				e = errno;
				kind = SHCLW_ERR_IO;
			}
			break;
		}
	}
	if (fclose(f) != 0 && kind == SHCLW_OK && buf) {
		e = errno;
		kind = SHCLW_ERR_IO;
	}
	if (!buf) {
		return shclw_i_fail(err, SHCLW_ERR_NO_MEMORY, path, NULL, 0);
	}
	if (kind != SHCLW_OK) {
		free(buf);
		return shclw_i_fail(err, kind, path, NULL, e);
	}
	if (!shclw_i_utf8_valid(buf, n)) {
		free(buf);
		return shclw_i_fail(err, SHCLW_ERR_IO, path, "not valid UTF-8", 0);
	}
	buf[n] = '\0';
	*text = buf;
	*len = n;
	return shclw_i_ok(err);
}
