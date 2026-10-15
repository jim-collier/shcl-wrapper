// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// The C API's own tests: what the shared cases can't see, such as who frees
// what, strings holding a NUL, and memory over many reads. The Makefile also
// builds this as C++.

#include "shclw.h"

#include <errno.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#if defined(__GLIBC__)
#include <malloc.h>
#endif

#if defined(__SANITIZE_ADDRESS__)
#define UNDER_ASAN 1
#elif defined(__has_feature)
#if __has_feature(address_sanitizer)
#define UNDER_ASAN 1
#endif
#endif

static int failures;

#define CHECK(cond)                                                                                                    \
	do {                                                                                                               \
		if (!(cond)) {                                                                                                 \
			(void)fprintf(stderr, "%s:%d: %s\n", __FILE__, __LINE__, #cond);                                           \
			failures++;                                                                                                \
		}                                                                                                              \
	} while (0)

static char scratch[4096];

static const char *in_scratch(const char *name) {
	static char path[4200];
	(void)snprintf(path, sizeof path, "%s/%s", scratch, name);
	return path;
}

static void put_file(const char *name, const char *text, size_t len) {
	FILE *f = fopen(in_scratch(name), "wb");
	CHECK(f != NULL);
	if (f) {
		CHECK(fwrite(text, 1, len, f) == len);
		CHECK(fclose(f) == 0);
	}
}

static shclw_doc *parse_z(const char *text) { return shclw_parse(text, strlen(text)); }

static void test_reads(void) {
	shclw_doc *d = parse_z("name: My App\nport: 8080\nratio: 0.25\ndebug: on\ntags: [red, green]\nports: [80, 443]\n");
	CHECK(d != NULL);

	shclw_got_int port = shclw_get_int(d, "port");
	CHECK(port.status == SHCLW_GOOD && port.value == 8080 && port.line == 2);
	shclw_got_float ratio = shclw_get_float(d, "ratio");
	CHECK(ratio.status == SHCLW_GOOD && ratio.value == 0.25);
	shclw_got_bool debug = shclw_get_bool(d, "debug");
	CHECK(debug.status == SHCLW_GOOD && debug.value);

	shclw_got_str name = shclw_get_str(d, "name");
	CHECK(name.status == SHCLW_GOOD && name.len == 6 && !strcmp(name.value, "My App") && name.line == 1);
	free(name.value);

	shclw_got_strs tags = shclw_get_strs(d, "tags");
	CHECK(tags.status == SHCLW_GOOD && tags.n == 2);
	if (tags.n == 2) {
		CHECK(!strcmp(tags.value[0].p, "red") && tags.value[1].n == 5);
	}
	free(tags.value);

	shclw_got_ints ports = shclw_get_ints(d, "ports");
	CHECK(ports.status == SHCLW_GOOD && ports.n == 2 && ports.value[1] == 443);
	free(ports.value);

	shclw_got_str missing = shclw_get_str(d, "missing");
	CHECK(missing.status == SHCLW_NOT_FOUND && missing.value && missing.len == 0 && missing.line == 0);
	free(missing.value);

	shclw_got_strs none = shclw_get_strs(d, "missing");
	CHECK(none.status == SHCLW_NOT_FOUND && none.value != NULL && none.n == 0);
	free(none.value);

	CHECK(shclw_int(d, "name", 7) == 7);
	CHECK(shclw_bool(d, "nope", true));
	CHECK(!strcmp(shclw_status_name(SHCLW_BAD_PATH), "BadPath"));
	CHECK(!strcmp(shclw_kind_name(SHCLW_ERR_NO_READ_BACK), "noreadback"));
	shclw_free(d);
	shclw_free(NULL);
}

static void test_fallbacks(void) {
	shclw_doc *d = parse_z("big: 1e400\nbigs: [1, 1e400]\n");
	shclw_got_float big = shclw_get_float(d, "big");
	CHECK(big.status == SHCLW_BAD_TYPE && big.value == 0);
	CHECK(shclw_float(d, "big", 1.5) == 1.5);
	shclw_got_floats bigs = shclw_get_floats(d, "bigs");
	CHECK(bigs.status == SHCLW_BAD_TYPE && bigs.n == 2 && bigs.value[0] == 1 && bigs.value[1] == 0);
	free(bigs.value);

	size_t len = 99;
	char *s = shclw_str(d, "missing", NULL, 0, &len);
	CHECK(s && len == 0 && s[0] == '\0');
	free(s);
	s = shclw_str(d, "missing", "a\0b", 3, &len);
	CHECK(s && len == 3 && s[1] == '\0' && s[2] == 'b' && s[3] == '\0');
	free(s);

	// The fallback comes back as a copy, so the caller's array can go.
	char word[] = "kept";
	shclw_string fall[2];
	fall[0].p = word;
	fall[0].n = 4;
	fall[1].p = "";
	fall[1].n = 0;
	size_t n = 0;
	shclw_string *v = shclw_strs(d, "missing", fall, 2, &n);
	word[0] = 'X';
	CHECK(v && n == 2 && !strcmp(v[0].p, "kept") && v[1].n == 0 && v[1].p[0] == '\0');
	free(v);

	double ffall[1] = {2.5};
	double *f = shclw_floats(d, "bigs", ffall, 1, &n);
	CHECK(f && n == 1 && f[0] == 2.5);
	free(f);
	int64_t *ints = shclw_ints(d, "missing", NULL, 0, &n);
	CHECK(ints != NULL && n == 0);
	free(ints);
	shclw_free(d);
}

// 2026101513330047: a value can hold a NUL, and the text and the save keep
// every byte after one.
static void test_nul(void) {
	shclw_doc *d = parse_z("k: \"a\xE2\x97\x89NUL\xE2\x97\x89z\"\nl: [x, \"y\xE2\x97\x89NUL\xE2\x97\x89\"]\n");
	shclw_got_str k = shclw_get_str(d, "k");
	CHECK(k.status == SHCLW_GOOD && k.len == 3 && k.value[1] == '\0' && k.value[2] == 'z' && k.value[3] == '\0');
	free(k.value);
	shclw_got_strs l = shclw_get_strs(d, "l");
	CHECK(l.status == SHCLW_GOOD && l.n == 2 && l.value[1].n == 2 && l.value[1].p[1] == '\0');
	free(l.value);
	shclw_free(d);

	static const char text[] = "# a\0b\nk: 1\n";
	d = shclw_parse(text, sizeof text - 1);
	size_t len = 0;
	char *out = shclw_text(d, &len);
	CHECK(out && len == sizeof text - 1 && !memcmp(out, text, len));
	free(out);
	CHECK(shclw_write(d, in_scratch("nul.shcl"), NULL) == SHCLW_OK);
	char *saved = NULL;
	CHECK(shclw_read_text(in_scratch("nul.shcl"), &saved, &len, NULL) == SHCLW_OK);
	CHECK(saved && len == sizeof text - 1 && !memcmp(saved, text, len));
	free(saved);
	shclw_free(d);
}

// 2026101513330046: shcl keeps what each read hands out until the document is
// freed. The wrapper copies it out and gives it back, so reads don't grow.
static void test_reads_give_back(void) {
#if defined(__GLIBC__) && !defined(UNDER_ASAN)
	shclw_doc *d = parse_z("tags: [alpha, beta, gamma, delta, epsilon]\nname: \"some text here\"\n");
	// The first pass sets up the block shcl keeps for its biggest read.
	struct mallinfo2 before = mallinfo2();
	for (int i = 0; i < 200001; i++) {
		if (i == 1) {
			before = mallinfo2();
		}
		shclw_got_strs g = shclw_get_strs(d, "tags");
		free(g.value);
		char *s = shclw_str(d, "tags", NULL, 0, NULL);
		free(s);
	}
	struct mallinfo2 after = mallinfo2();
	size_t grew = after.uordblks > before.uordblks ? after.uordblks - before.uordblks : 0;
	if (grew > (size_t)64 * 1024) {
		(void)fprintf(stderr, "200,000 reads kept %zu bytes\n", grew);
	}
	CHECK(grew <= (size_t)64 * 1024);
	shclw_free(d);
#else
	(void)puts("api: memory over many reads isn't measured in this build");
#endif
}

static void test_errors(void) {
	shclw_error err;
	shclw_doc *d = NULL;
	CHECK(shclw_open(in_scratch("missing.shcl"), &d, &err) == SHCLW_ERR_NOT_FOUND);
	CHECK(err.kind == SHCLW_ERR_NOT_FOUND && strstr(err.msg, "missing.shcl") && err.sys_errno == ENOENT);
	CHECK(d != NULL && shclw_int(d, "port", 9) == 9);
	shclw_free(d);

	put_file("bad.shcl", "k: \xff\n", 4);
	CHECK(shclw_open(in_scratch("bad.shcl"), &d, NULL) == SHCLW_ERR_IO);
	shclw_free(d);

	const char newer[] = "k: 1\n##    Format   9\n";
	put_file("newer.shcl", newer, sizeof newer - 1);
	CHECK(shclw_open(in_scratch("newer.shcl"), &d, &err) == SHCLW_ERR_TOO_NEW);
	CHECK(strstr(err.msg, "format 9") != NULL);
	CHECK(shclw_int(d, "k", 0) == 0);
	shclw_free(d);

	const char good[] = "k: 1\n";
	put_file("good.shcl", good, sizeof good - 1);
	CHECK(shclw_open(in_scratch("good.shcl"), &d, &err) == SHCLW_OK && err.kind == SHCLW_OK);
	CHECK(shclw_int(d, "k", 0) == 1);
	CHECK(shclw_write(d, in_scratch("no/such/dir/x.shcl"), &err) == SHCLW_ERR_NOT_FOUND);
	shclw_free(d);

	const char bad[] = "a: 1\n1x y: 2\n";
	CHECK(shclw_parse_with(bad, sizeof bad - 1, SHCLW_STRICT, &d, &err) == SHCLW_ERR_STRICT);
	CHECK(d != NULL && shclw_errors(d) == 1);
	size_t n = 0;
	shclw_diag *ds = shclw_diags(d, &n);
	CHECK(ds && n == 1 && ds[0].line == 2 && ds[0].severity == SHCLW_SEV_ERROR && ds[0].code.n == 4);
	if (ds && n == 1) {
		CHECK(!strcmp(ds[0].code.p, "E014") && ds[0].message.n == strlen(ds[0].message.p));
	}
	free(ds);
	shclw_free(d);
	put_file("strict.shcl", bad, sizeof bad - 1);
	CHECK(shclw_open_with(in_scratch("strict.shcl"), SHCLW_STRICT, &d, &err) == SHCLW_ERR_STRICT);
	CHECK(strstr(err.msg, "strict.shcl") != NULL);
	shclw_free(d);

	// A save that would drop a line the load couldn't use, since an edit
	// made the kept lines impossible.
	const char lossy[] = "a: 1\na(5): 2\n";
	d = shclw_parse(lossy, sizeof lossy - 1);
	CHECK(shcl_remove(shclw_inner(d), "a", 1) == 1);
	CHECK(shclw_write(d, in_scratch("lossy.shcl"), &err) == SHCLW_ERR_REFUSED);
	CHECK(access(in_scratch("lossy.shcl"), F_OK) != 0);
	shclw_free(d);

	d = shclw_new();
	ds = shclw_diags(d, &n);
	CHECK(ds != NULL && n == 0);
	free(ds);
	shclw_free(d);
}

static void cleanup(void) {
	static const char *const made[] = {"nul.shcl", "bad.shcl", "newer.shcl", "good.shcl", "strict.shcl"};
	for (size_t i = 0; i < sizeof made / sizeof made[0]; i++) {
		if (unlink(in_scratch(made[i])) != 0 && errno != ENOENT) {
			(void)fprintf(stderr, "couldn't remove %s\n", in_scratch(made[i]));
		}
	}
	if (rmdir(scratch) != 0) {
		(void)fprintf(stderr, "couldn't remove %s\n", scratch);
	}
}

int main(void) {
	const char *tmp = getenv("TMPDIR");
	(void)snprintf(scratch, sizeof scratch, "%s/shclw-c-api-XXXXXX", tmp && *tmp ? tmp : "/tmp");
	if (!mkdtemp(scratch)) {
		perror("mkdtemp");
		return 2;
	}
	test_reads();
	test_fallbacks();
	test_nul();
	test_reads_give_back();
	test_errors();
	cleanup();
	if (failures) {
		(void)fprintf(stderr, "api: %d failed\n", failures);
		return 1;
	}
	(void)puts("api: ok");
	return 0;
}
