#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

tools_manifest_path_arg := --manifest-path=tool/Cargo.toml

run := cargo run \
	$(tools_manifest_path_arg) -p manage-cargo-manifests --

run_allow := cargo run \
	$(tools_manifest_path_arg) -p manage-direct-dependency-allow-list --

project_manifest_path_arg := --manifest-path=../../Cargo.toml

allowlist := direct-dependency-allow-list.toml

run_allow_check_workspace := $(run_allow) check-workspace --allowlist $(allowlist) $(project_manifest_path_arg)
run_allow_update := $(run_allow) update --allowlist $(allowlist) -o $(allowlist)

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
	$(run_allow_check_workspace)

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check
	$(run_allow_check_workspace)

.PHONY: check-workspace-direct-dependencies
check-workspace-direct-dependencies:
	$(run_allow_check_workspace)

.PHONY: update-workspace-direct-dependencies
update-workspace-direct-dependencies:
	$(run_allow_update)

.PHONY: dry-update-workspace-direct-dependencies
dry-update-workspace-direct-dependencies:
	$(run_allow_update) --dry-run
