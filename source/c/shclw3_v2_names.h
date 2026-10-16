// Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
// SPDX-License-Identifier: MIT

// shcl 2.0's calls under names of their own, so its shcl.h can be built into a
// program that also has 3.0's. Each of 2.0's external symbols is here; the
// Makefile's names check fails on one that isn't. Only for shclw3_v2.c and
// shclw3_v2_impl.c, ahead of 2.0's shcl.h.

#ifndef SHCLW3_V2_NAMES_H
#define SHCLW3_V2_NAMES_H

// The upgrade check only parses, and this keeps 2.0's file calls out.
#define SHCL_NO_FILE_IO

// clang-format off
#define shcl_authored_name                   shclw_v2_authored_name
#define shcl_children                        shclw_v2_children
#define shcl_count                           shclw_v2_count
#define shcl_datetime_str                    shclw_v2_datetime_str
#define shcl_diag_code                       shclw_v2_diag_code
#define shcl_diag_count                      shclw_v2_diag_count
#define shcl_diag_line                       shclw_v2_diag_line
#define shcl_diag_message                    shclw_v2_diag_message
#define shcl_diag_severity                   shclw_v2_diag_severity
#define shcl_error_count                     shclw_v2_error_count
#define shcl_exists                          shclw_v2_exists
#define shcl_format_f64                      shclw_v2_format_f64
#define shcl_free                            shclw_v2_free
#define shcl_generate                        shclw_v2_generate
#define shcl_get_bool                        shclw_v2_get_bool
#define shcl_get_bool_or                     shclw_v2_get_bool_or
#define shcl_get_float                       shclw_v2_get_float
#define shcl_get_float_or                    shclw_v2_get_float_or
#define shcl_get_int                         shclw_v2_get_int
#define shcl_get_int_or                      shclw_v2_get_int_or
#define shcl_instances                       shclw_v2_instances
#define shcl_line                            shclw_v2_line
#define shcl_lines                           shclw_v2_lines
#define shcl_load_and_validate               shclw_v2_load_and_validate
#define shcl_lost_count                      shclw_v2_lost_count
#define shcl_merge                           shclw_v2_merge
#define shcl_new                             shclw_v2_new
#define shcl_parse                           shclw_v2_parse
#define shcl_parse_with                      shclw_v2_parse_with
#define shcl_paths                           shclw_v2_paths
#define shcl_quote_segment                   shclw_v2_quote_segment
#define shcl_quoted                          shclw_v2_quoted
#define shcl_read_bool_                      shclw_v2_read_bool_
#define shcl_read_bool_array                 shclw_v2_read_bool_array
#define shcl_read_datetime                   shclw_v2_read_datetime
#define shcl_read_datetime_array             shclw_v2_read_datetime_array
#define shcl_read_float                      shclw_v2_read_float
#define shcl_read_float_array                shclw_v2_read_float_array
#define shcl_read_int                        shclw_v2_read_int
#define shcl_read_int_array                  shclw_v2_read_int_array
#define shcl_read_raw                        shclw_v2_read_raw
#define shcl_read_raw_info                   shclw_v2_read_raw_info
#define shcl_read_string                     shclw_v2_read_string
#define shcl_read_string_array               shclw_v2_read_string_array
#define shcl_remove                          shclw_v2_remove
#define shcl_set_bool                        shclw_v2_set_bool
#define shcl_set_bool_array                  shclw_v2_set_bool_array
#define shcl_set_bool_array_default          shclw_v2_set_bool_array_default
#define shcl_set_bool_default                shclw_v2_set_bool_default
#define shcl_set_comment                     shclw_v2_set_comment
#define shcl_set_datetime                    shclw_v2_set_datetime
#define shcl_set_datetime_array              shclw_v2_set_datetime_array
#define shcl_set_datetime_array_default      shclw_v2_set_datetime_array_default
#define shcl_set_datetime_default            shclw_v2_set_datetime_default
#define shcl_set_empty                       shclw_v2_set_empty
#define shcl_set_float                       shclw_v2_set_float
#define shcl_set_float_array                 shclw_v2_set_float_array
#define shcl_set_float_array_default         shclw_v2_set_float_array_default
#define shcl_set_float_default               shclw_v2_set_float_default
#define shcl_set_int                         shclw_v2_set_int
#define shcl_set_int_array                   shclw_v2_set_int_array
#define shcl_set_int_array_default           shclw_v2_set_int_array_default
#define shcl_set_int_default                 shclw_v2_set_int_default
#define shcl_set_literal                     shclw_v2_set_literal
#define shcl_set_literal_default             shclw_v2_set_literal_default
#define shcl_set_raw                         shclw_v2_set_raw
#define shcl_set_raw_default                 shclw_v2_set_raw_default
#define shcl_set_string                      shclw_v2_set_string
#define shcl_set_string_array                shclw_v2_set_string_array
#define shcl_set_string_array_default        shclw_v2_set_string_array_default
#define shcl_set_string_default              shclw_v2_set_string_default
#define shcl_status_code                     shclw_v2_status_code
#define shcl_status_name                     shclw_v2_status_name
#define shcl_status_ok                       shclw_v2_status_ok
#define shcl_strict_failed                   shclw_v2_strict_failed
#define shcl_strictness_from_arg             shclw_v2_strictness_from_arg
#define shcl_strictness_of                   shclw_v2_strictness_of
#define shcl_suppress_declared_reopens       shclw_v2_suppress_declared_reopens
#define shcl_suppress_declared_repeats       shclw_v2_suppress_declared_repeats
#define shcl_to_canonical                    shclw_v2_to_canonical
#define shcl_validate                        shclw_v2_validate
#define shcl_validation_code                 shclw_v2_validation_code
#define shcl_validation_count                shclw_v2_validation_count
#define shcl_validation_free                 shclw_v2_validation_free
#define shcl_validation_line                 shclw_v2_validation_line
#define shcl_validation_message              shclw_v2_validation_message
#define shcl_validation_severity             shclw_v2_validation_severity
#define shcl_write_reason_                   shclw_v2_write_reason_
// clang-format on

#endif
