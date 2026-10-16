// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// The 3.0 upgrade's check against shcl 2.0 (design.md, Format and upgrade).
// Built with 2.0's shcl.h on the include path, and not 3.0's.

#include "shclw3_v2_names.h"

#include "shcl.h"

#include "shclw3_v2.h"

#ifdef SHCL_FORMAT_MAJOR
#error "shclw3_v2.c wants shcl 2.0's shcl.h, and this one is 3.0 or newer"
#endif

size_t shclw_i_v2_errors(const char *text, size_t len, shclw_i_v2_each each, void *ctx) {
	shcl_doc *d = shcl_parse(text, len);
	size_t found = 0;
	for (size_t i = 0; i < shcl_diag_count(d); i++) {
		if (shcl_diag_severity(d, i) != SHCL_SEV_ERROR) {
			continue;
		}
		shcl_str msg = shcl_diag_message(d, i);
		each(ctx, shcl_diag_line(d, i), shcl_diag_code(d, i), msg.p, msg.n);
		found++;
	}
	shcl_free(d);
	return found;
}
