#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

run := cargo run \
	--manifest-path=tool/Cargo.toml -p manage-cargo-manifests --

run_allow := cargo run \
	--manifest-path=tool/Cargo.toml -p manage-direct-dependency-allow-list --

run_allow_check_workspace := $(run_allow) check-workspace \
	--manifest-path=../../Cargo.toml --allowlist direct-dependency-allow-list.toml

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf tool/target $(blueprint)

.PHONY: $(blueprint)
$(blueprint):
	nix-build -A blueprintJSON --out-link $@

.PHONY: update
update: $(blueprint)
	$(run) --blueprint $<
	$(run_allow)

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check
	$(run_allow)

.PHONY: check-workspace-direct-dependencies
check-workspace-direct-dependencies:
	$(run_allow_check_workspace)
