#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

expr_path = ../..

attr_args := -A everythingWithExcess

nix_build_cmd := nix-build $(expr_path) $(attr_args) -j1 --no-out-link

cache_name := coliasgroup

.PHONY: push
push:
	nix-store -qR --include-outputs $$(nix-store -qd $$($(nix_build_cmd))) \
		| grep -v '\.drv$$' \
		| cachix push $(cache_name)
