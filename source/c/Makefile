# Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
# SPDX-License-Identifier: MIT
#
# Builds and tests the C wrapper. SHCL_DIR is the folder holding the shcl.h
# to build against, and SHCL2_DIR the one with shcl 2.0's, which the 3.0
# upgrade checks old files with. cicd/cicd.bash fetches the pinned ones.
#
#   make SHCL_DIR=... SHCL2_DIR=... test     API tests: sanitized, plain, and built as C++
#   make SHCL_DIR=... SHCL2_DIR=... drive    the sanitized shclw-drive, for the shared cases
#   make SHCL_DIR=... SHCL2_DIR=... strict   compile only, with gcc-15 at C17
#   make SHCL_DIR=... SHCL2_DIR=... lint     clang-tidy
#   make format-check | format

SHCL_DIR ?=
SHCL2_DIR ?=
ifneq ($(filter-out format format-check clean,$(or $(MAKECMDGOALS),all)),)
ifeq ($(SHCL_DIR),)
$(error set SHCL_DIR to the folder with shcl.h)
endif
ifeq ($(SHCL2_DIR),)
$(error set SHCL2_DIR to the folder with shcl 2.0's shcl.h)
endif
endif
OUT      ?= build
CC       ?= gcc
CXX      ?= g++
CC15     ?= gcc-15

LIB   := shclw.c info.c shclw3.c run.c
WARN  := -Wall -Wextra -Wpedantic -Wshadow -Wvla -Wconversion -Werror
CFLAGS_ALL := -std=c11 -O1 -g $(WARN) -I. -I$(SHCL_DIR)
CXXFLAGS_ALL := -std=c++20 -O1 -g -Wall -Wextra -Wpedantic -Wshadow -Wconversion -Werror -I. -I$(SHCL_DIR)
SAN   := -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer
# shcl is first-party in its own repo; here it's built as a program would.
SHCL_FLAGS := -std=c11 -O1 -g -I$(SHCL_DIR)
# shcl 2.0, renamed, for the upgrade check. Its 2 files see 2.0's shcl.h and
# never 3.0's.
V2_FLAGS := -std=c11 -O1 -g $(WARN) -I. -I$(SHCL2_DIR)
V2_CXXFLAGS := -std=c++20 -O1 -g -Wall -Wextra -Wpedantic -Wshadow -Wconversion -Werror -I. -I$(SHCL2_DIR)
SHCL2_FLAGS := -std=c11 -O1 -g -I$(SHCL2_DIR)
V2_DEPS := shclw3_v2.h shclw3_v2_names.h $(SHCL2_DIR)/shcl.h

FORMATTED := $(wildcard *.c *.h tests/*.c)

.PHONY: all test drive strict lint names format format-check clean
all: test drive strict

$(OUT)/asan $(OUT)/plain $(OUT)/cxx:
	mkdir -p $@

$(OUT)/asan/%.o: %.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/asan
	$(CC) $(CFLAGS_ALL) $(SAN) -c $< -o $@
$(OUT)/asan/%.o: tests/%.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/asan
	$(CC) $(CFLAGS_ALL) $(SAN) -c $< -o $@
$(OUT)/asan/shcl_impl.o: tests/shcl_impl.c $(SHCL_DIR)/shcl.h | $(OUT)/asan
	$(CC) $(SHCL_FLAGS) $(SAN) -c $< -o $@

$(OUT)/plain/%.o: %.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/plain
	$(CC) $(CFLAGS_ALL) -c $< -o $@
$(OUT)/plain/%.o: tests/%.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/plain
	$(CC) $(CFLAGS_ALL) -c $< -o $@
$(OUT)/plain/shcl_impl.o: tests/shcl_impl.c $(SHCL_DIR)/shcl.h | $(OUT)/plain
	$(CC) $(SHCL_FLAGS) -c $< -o $@

$(OUT)/asan/shclw3_v2.o: shclw3_v2.c $(V2_DEPS) | $(OUT)/asan
	$(CC) $(V2_FLAGS) $(SAN) -c $< -o $@
$(OUT)/asan/shclw3_v2_impl.o: shclw3_v2_impl.c $(V2_DEPS) | $(OUT)/asan
	$(CC) $(SHCL2_FLAGS) $(SAN) -c $< -o $@
$(OUT)/plain/shclw3_v2.o: shclw3_v2.c $(V2_DEPS) | $(OUT)/plain
	$(CC) $(V2_FLAGS) -c $< -o $@
$(OUT)/plain/shclw3_v2_impl.o: shclw3_v2_impl.c $(V2_DEPS) | $(OUT)/plain
	$(CC) $(SHCL2_FLAGS) -c $< -o $@
$(OUT)/cxx/shclw3_v2.o: shclw3_v2.c $(V2_DEPS) | $(OUT)/cxx
	$(CXX) -x c++ $(V2_CXXFLAGS) -c $< -o $@

# Every C file of ours again as C++, which is how a C++ program sees shclw.h.
$(OUT)/cxx/%.o: %.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/cxx
	$(CXX) -x c++ $(CXXFLAGS_ALL) -c $< -o $@
$(OUT)/cxx/%.o: tests/%.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/cxx
	$(CXX) -x c++ $(CXXFLAGS_ALL) -c $< -o $@

objs = $(addprefix $(OUT)/$(1)/,$(LIB:.c=.o) shclw3_v2.o)

$(OUT)/shclw-drive: $(call objs,asan) $(OUT)/asan/shclw-drive.o $(OUT)/asan/shcl_impl.o $(OUT)/asan/shclw3_v2_impl.o
	$(CC) $(SAN) $^ -lm -o $@
$(OUT)/api-test-asan: $(call objs,asan) $(OUT)/asan/api_test.o $(OUT)/asan/shcl_impl.o $(OUT)/asan/shclw3_v2_impl.o
	$(CC) $(SAN) $^ -lm -o $@
$(OUT)/api-test-plain: $(call objs,plain) $(OUT)/plain/api_test.o $(OUT)/plain/shcl_impl.o $(OUT)/plain/shclw3_v2_impl.o
	$(CC) $^ -lm -o $@
$(OUT)/api-test-cxx: $(call objs,cxx) $(OUT)/cxx/api_test.o $(OUT)/plain/shcl_impl.o $(OUT)/plain/shclw3_v2_impl.o
	$(CXX) $^ -lm -o $@

drive: $(OUT)/shclw-drive

test: names $(OUT)/api-test-asan $(OUT)/api-test-plain $(OUT)/api-test-cxx
	$(OUT)/api-test-asan
	$(OUT)/api-test-plain
	$(OUT)/api-test-cxx

# A 2.0 name left as it was would meet 3.0's at link time, or, with no 3.0
# twin, hide in the program as shcl's.
names: $(OUT)/plain/shclw3_v2.o $(OUT)/plain/shclw3_v2_impl.o
	@if nm -g --defined-only $^ | grep -E ' _?shcl_'; then echo "shcl 2.0 names not in shclw3_v2_names.h" >&2; exit 1; fi

strict:
	$(CC15) -std=c17 $(WARN) -I. -I$(SHCL_DIR) -fsyntax-only $(LIB) shclw-drive.c tests/api_test.c
	$(CC15) -std=c17 $(WARN) -I. -I$(SHCL2_DIR) -fsyntax-only shclw3_v2.c

lint:
	clang-tidy --quiet $(LIB) shclw-drive.c tests/api_test.c -- -std=c11 -I. -I$(SHCL_DIR)
	clang-tidy --quiet shclw3_v2.c -- -std=c11 -I. -I$(SHCL2_DIR)

format:
	clang-format -i $(FORMATTED)
format-check:
	clang-format --dry-run -Werror $(FORMATTED)

clean:
	rm -rf -- $(OUT)
