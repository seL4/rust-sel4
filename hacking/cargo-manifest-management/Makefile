#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

project_root := ../..

run_script := \
	script=$$(nix-build -A script --no-out-link) && $$script --root $(project_root)

.PHONY: none
none:

.PHONY: update
update:
	$(run_script)

.PHONY: check
check:
	$(run_script) --just-check
