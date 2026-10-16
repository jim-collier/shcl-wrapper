// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// shcl 2.0 in a 3.0 build, for the upgrade check. This side names no shcl type,
// since 2.0's shcl.h and 3.0's can't both be in one .c file.

#ifndef SHCLW3_V2_H
#define SHCLW3_V2_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Called once per error, in shcl 2.0's order. code is NUL-terminated; msg
// has its length.
typedef void (*shclw_i_v2_each)(void *ctx, size_t line, const char *code, const char *msg, size_t msg_len);

// The errors shcl 2.0 reports on text, through each, and how many. 2.0 ends
// the program when it runs out of memory, as it always has.
size_t shclw_i_v2_errors(const char *text, size_t len, shclw_i_v2_each each, void *ctx);

#ifdef __cplusplus
}
#endif

#endif
