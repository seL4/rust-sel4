#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

top_level_dir := $(TOP_LEVEL_DIR)

build_dir := build

dafny_link := $(build_dir)/dafny
dafny_bin := $(dafny_link)/bin/dafny

dafny_name := core
dafny_src := ../verified/$(dafny_name).dfy
dafny_output_prefix := $(build_dir)/$(dafny_name)
dafny_output := $(dafny_output_prefix)-rust/src/$(dafny_name).rs
dafny_dst := $(build_dir)/translated.rs

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(build_dir)

FORCE: ;

$(build_dir):
	mkdir -p $@

$(dafny_link): FORCE | $(build_dir)
	nix-build $(top_level_dir) -A pkgs.build.this.dafny -o $@

$(dafny_output): FORCE | $(build_dir)
	$(dafny_bin) translate rs $(dafny_src) -o $(dafny_output_prefix)

$(dafny_dst): $(dafny_output)
	cp $< $@

# # #

.PHONY: dafny
dafny: $(dafny_link)

.PHONY: translate
translate: $(dafny_dst)

.PHONY: test
test:
	cargo test -p tests-root-task-dafny-core

.PHONY: run
run:
	set -eu; \
	script=$$( \
		nix-build $(top_level_dir) \
			-A worlds.aarch64.default.instances.tests.root-task.dafny.automate \
			--no-out-link \
	); \
	$$script
