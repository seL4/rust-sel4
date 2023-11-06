#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := $$(nix-build -A blueprintJSON --no-out-link)

target_dir := target

bin_dir := $(target_dir)/debug

run := PATH=$(bin_dir):$$PATH $(bin_dir)/manage-cargo-manifests --blueprint $(blueprint)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(target_dir)

.INTERMEDIATE: bins
bins:
	cargo build

.PHONY: update
update: bins
	$(run)

.PHONY: check
check: bins
	$(run) --just-check
