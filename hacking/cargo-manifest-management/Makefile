#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

plan := $$(nix-build -A planJSON --no-out-link)
run := cargo run -- --plan $(plan)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf target

.PHONY: update
update:
	$(run)

.PHONY: check
check:
	$(run) --just-check
