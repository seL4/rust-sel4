#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

run_script := \
	script=$$(nix-build -A script --no-out-link) && $$script

.PHONY: none
none:

.PHONY: update
update:
	$(run_script)

.PHONY: check
check:
	$(run_script) --just-check
