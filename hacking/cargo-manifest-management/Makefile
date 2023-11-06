#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf target

.PHONY: update
update:
	script=$$(nix-build -A updateScript --no-out-link) && $$script

.PHONY: check
check:
	nix-build -A witness --no-out-link
