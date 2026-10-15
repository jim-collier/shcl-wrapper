# Copyright © 2026 Jim Collier [ID: 2უNაɘ«҂թȹɤξπ๙¿ձϖ]
# SPDX-License-Identifier: MIT
#
# Builds and tests the C wrapper. SHCL_DIR is the folder holding the shcl.h
# to build against; cicd/cicd.bash fetches the pinned one.
#
#   make SHCL_DIR=... test     API tests: sanitized, plain, and built as C++
#   make SHCL_DIR=... drive    the sanitized shclw-drive, for the shared cases
#   make SHCL_DIR=... strict   compile only, with gcc-15 at C17
#   make SHCL_DIR=... lint     clang-tidy
#   make format-check | format

SHCL_DIR ?=
ifeq ($(SHCL_DIR),)
ifneq ($(filter-out format format-check clean,$(or $(MAKECMDGOALS),all)),)
$(error set SHCL_DIR to the folder with shcl.h)
endif
endif
OUT      ?= build
CC       ?= gcc
CXX      ?= g++
CC15     ?= gcc-15

LIB   := shclw.c shclw3.c run.c
WARN  := -Wall -Wextra -Wpedantic -Wshadow -Wvla -Wconversion -Werror
CFLAGS_ALL := -std=c11 -O1 -g $(WARN) -I. -I$(SHCL_DIR)
CXXFLAGS_ALL := -std=c++20 -O1 -g -Wall -Wextra -Wpedantic -Wshadow -Wconversion -Werror -I. -I$(SHCL_DIR)
SAN   := -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer
# shcl is first-party in its own repo; here it's built as a program would.
SHCL_FLAGS := -std=c11 -O1 -g -I$(SHCL_DIR)

FORMATTED := $(wildcard *.c *.h tests/*.c)

.PHONY: all test drive strict lint format format-check clean
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

# Every C file of ours again as C++, which is how a C++ program sees shclw.h.
$(OUT)/cxx/%.o: %.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/cxx
	$(CXX) -x c++ $(CXXFLAGS_ALL) -c $< -o $@
$(OUT)/cxx/%.o: tests/%.c *.h $(SHCL_DIR)/shcl.h | $(OUT)/cxx
	$(CXX) -x c++ $(CXXFLAGS_ALL) -c $< -o $@

objs = $(addprefix $(OUT)/$(1)/,$(LIB:.c=.o))

$(OUT)/shclw-drive: $(call objs,asan) $(OUT)/asan/shclw-drive.o $(OUT)/asan/shcl_impl.o
	$(CC) $(SAN) $^ -lm -o $@
$(OUT)/api-test-asan: $(call objs,asan) $(OUT)/asan/api_test.o $(OUT)/asan/shcl_impl.o
	$(CC) $(SAN) $^ -lm -o $@
$(OUT)/api-test-plain: $(call objs,plain) $(OUT)/plain/api_test.o $(OUT)/plain/shcl_impl.o
	$(CC) $^ -lm -o $@
$(OUT)/api-test-cxx: $(call objs,cxx) $(OUT)/cxx/api_test.o $(OUT)/plain/shcl_impl.o
	$(CXX) $^ -lm -o $@

drive: $(OUT)/shclw-drive

test: $(OUT)/api-test-asan $(OUT)/api-test-plain $(OUT)/api-test-cxx
	$(OUT)/api-test-asan
	$(OUT)/api-test-plain
	$(OUT)/api-test-cxx

strict:
	$(CC15) -std=c17 $(WARN) -I. -I$(SHCL_DIR) -fsyntax-only $(LIB) shclw-drive.c tests/api_test.c

lint:
	clang-tidy --quiet $(LIB) shclw-drive.c tests/api_test.c -- -std=c11 -I. -I$(SHCL_DIR)

format:
	clang-format -i $(FORMATTED)
format-check:
	clang-format --dry-run -Werror $(FORMATTED)

clean:
	rm -rf -- $(OUT)
