#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

P ?= ../..
A ?= everything

path := $(P)
attr := $(A)

nix_build_cmd := nix-build $(path) -A $(attr) --no-out-link

cache_name := coliasgroup

.PHONY: push
push:
	nix-store -qR --include-outputs $$(nix-store -qd $$($(nix_build_cmd))) \
		| grep -v '\.drv$$' \
		| cachix push $(cache_name)
