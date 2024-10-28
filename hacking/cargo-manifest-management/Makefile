#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

run := cargo run --manifest-path=tool/Cargo.toml -p manage-cargo-manifests --

run_check_dependencies := \
	cargo run --manifest-path=tool/Cargo.toml -p check-dependency-allow-list -- \
		--manifest-path=../../Cargo.toml --allowlist third-party-dependency-versions.toml

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
	$(run_check_dependencies)

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check
	$(run_check_dependencies)

.PHONY: check-dependencies
check-dependencies:
	$(run_check_dependencies)
