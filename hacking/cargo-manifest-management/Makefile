#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

run := cargo run -p manage-cargo-manifests --

run_allowlist := cargo run -p manage-direct-dependency-allow-list --

project_manifest_path_arg := --manifest-path=../../Cargo.toml

allowlist := direct-dependency-allow-list.toml

run_allowlist_check_workspace := $(run_allowlist) check-workspace --allowlist $(allowlist) $(project_manifest_path_arg)
run_allowlist_update := $(run_allowlist) update --allowlist $(allowlist) -o $(allowlist)

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

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check

.PHONY: check-workspace-direct-dependencies
check-workspace-direct-dependencies:
	$(run_allowlist_check_workspace)

.PHONY: update-direct-dependency-allowlist
update-direct-dependency-allowlist:
	$(run_allowlist_update)

.PHONY: check-direct-dependency-allow-list
check-direct-dependency-allow-list:
	$(run_allowlist_update) --check
