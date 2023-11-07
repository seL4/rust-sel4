#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

run := cargo run --manifest-path=tool/Cargo.toml -p manage-cargo-manifests --

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(target_dir) $(blueprint)

.PHONY: $(blueprint)
$(blueprint):
	nix-build -A blueprintJSON --out-link $@

.PHONY: update
update: $(blueprint)
	$(run) --blueprint $<

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check
