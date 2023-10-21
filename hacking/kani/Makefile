#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

BUILD ?= .

build_dir := $(BUILD)
target_dir := $(build_dir)/target

crates := \
	sel4-bitfield-ops

.PHONY: none
none:

.PHONY: check
check:
	cargo kani --target-dir=$(abspath $(target_dir)) $(addprefix -p,$(crates))
