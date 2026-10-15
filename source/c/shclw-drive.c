// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// shclw-drive runs an ops script against a document, for the tests that
// compare wrapper languages. The same command line as the Go and Rust ones:
//
//	shclw-drive VERSION DOCFILE [DIR] < SCRIPT
//
// C picks its shcl when it's built, so VERSION has to be that one.

#include "shclw_adapter.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int fail(const char *why, const char *what) {
	(void)fprintf(stderr, "shclw-drive: %s%s\n", why, what ? what : "");
	return 2;
}

static char *read_all(FILE *f, size_t *len) {
	size_t cap = 4096;
	size_t n = 0;
	char *buf = (char *)malloc(cap);
	while (buf) {
		if (n == cap) {
			char *grown = (char *)realloc(buf, cap * 2);
			if (!grown) {
				free(buf);
				return NULL;
			}
			buf = grown;
			cap *= 2;
		}
		size_t want = cap - n;
		size_t got = fread(buf + n, 1, want, f);
		n += got;
		if (got < want) {
			if (ferror(f)) {
				free(buf);
				return NULL;
			}
			break;
		}
	}
	*len = n;
	return buf;
}

int main(int argc, char **argv) {
	if (argc < 3 || argc > 4) {
		return fail("usage: shclw-drive VERSION DOCFILE [DIR] < SCRIPT", NULL);
	}
	char version[8];
	(void)snprintf(version, sizeof version, "%d", SHCLW_SHCL);
	if (strcmp(argv[1], version) != 0) {
		return fail("built for shcl version ", version);
	}
	char *doc = NULL;
	size_t doc_len = 0;
	shclw_error err;
	if (shclw_read_text(argv[2], &doc, &doc_len, &err) != SHCLW_OK) {
		return fail(err.msg, NULL);
	}
	size_t script_len = 0;
	char *script = read_all(stdin, &script_len);
	if (!script || !shclw_i_utf8_valid(script, script_len)) {
		free(doc);
		free(script);
		return fail("script is not valid UTF-8, or couldn't be read", NULL);
	}
	size_t out_len = 0;
	char *out = shclw_run(doc, doc_len, script, script_len, argc == 4 ? argv[3] : NULL, &out_len);
	free(doc);
	free(script);
	if (!out) {
		return fail("out of memory", NULL);
	}
	int rc = fwrite(out, 1, out_len, stdout) == out_len && fflush(stdout) == 0 ? 0 : 1;
	free(out);
	return rc;
}
