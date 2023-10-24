#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

ifeq ($(K),1)
	keep_going := -k
endif

ifneq ($(J),)
	jobs_arg := -j$(J)
else
	jobs_arg := -j$$(nproc)
endif

ifneq ($(CORES),)
	cores_arg := --cores $(CORES)
else
	cores_arg :=
endif

out := out

nix_build := nix-build $(keep_going) $(jobs_arg) $(cores_arg)

nix_shell := nix-shell -A shell --pure

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(out) target

$(out):
	mkdir -p $@

rustc_target_spec_dir := support/targets

.PHONY: generate-target-specs
generate-target-specs:
	rm -f $(rustc_target_spec_dir)/*.json && \
		cargo run -p sel4-generate-target-specs -- write --target-dir $(rustc_target_spec_dir) --all

.PHONY: update-generated-sources
update-generated-sources:
	$(MAKE) -C hacking/cargo-manifest-management update
	cargo update -w

.PHONY: check-generated-sources
check-generated-sources:
	$(MAKE) -C hacking/cargo-manifest-management check
	cargo update -w --locked

.PHONY: update-lockfile
update-lockfile:
	cargo update -w

.PHONY: check-lockfile
check-lockfile:
	cargo update -w --locked

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: check-fmt
check-fmt:
	cargo fmt --all -- --check

.PHONY: check-generic-formatting
check-generic-formatting:
	$(nix_shell) --run "sh hacking/scripts/check-generic-formatting.sh"

.PHONY: check-source
check-source: check-generated-sources check-fmt check-generic-formatting

.PHONY: check-licenses
check-licenses:
	$(nix_shell) --run "reuse lint"

.PHONY: check-dependencies
check-dependencies:
	lockfile=$$($(nix_build) -A pkgs.build.this.publicCratesCargoLock --no-out-link) && \
		$(nix_shell) --run "cargo-audit audit -f $$lockfile"

try_restore_terminal := tput smam 2> /dev/null || true

.PHONY: run-tests
run-tests:
	script=$$($(nix_build) -A runTests --no-out-link) && $$script
	$(try_restore_terminal)

.PHONY: run-sel4test-for
run-sel4test-for:
	# use trailing period to catch empty variable
	script=$$($(nix_build) -A sel4testInstances.$(SEL4TEST_ARCH). --no-out-link) && $$script
	$(try_restore_terminal)

.PHONY: run-fast-tests
run-fast-tests:
	script=$$($(nix_build) -A runFastTests --no-out-link) && $$script
	$(try_restore_terminal)

.PHONY: witness-tests
witness-tests:
	$(nix_build) -A witnessTests --no-out-link
	$(try_restore_terminal)

.PHONY: witness-fast-tests
witness-fast-tests:
	$(nix_build) -A witnessFastTests --no-out-link
	$(try_restore_terminal)

.PHONY: everything-except-non-incremental
everything-except-non-incremental:
	$(nix_build) -A everythingExceptNonIncremental --no-out-link

.PHONY: everything
everything:
	$(nix_build) -A everything --no-out-link

.PHONY: everything-with-excess
everything-with-excess:
	$(nix_build) -A everythingWithExcess --no-out-link

.PHONY: html-links
html-links:
	$(nix_build) -A html -o $(out)/$@

.PHONY: html
html: | $(out)
	src=$$($(nix_build) -A html --no-out-link) && \
	dst=$(out)/html && \
	rm -rf $$dst && \
	cp -rL --no-preserve=owner,mode $$src $$dst

.PHONY: example
example:
	script=$$($(nix_build) -A $@ --no-out-link) && $$script

.PHONY: example-rpi4-b-4gb
example-rpi4-b-4gb:
	$(nix_build) -A $@ -o $(out)/$@

.PHONY: check-immediately
check-immediately: check-source check-licenses check-dependencies

.PHONY: check-quickly
check-quickly: check-immediately
	$(MAKE) witness-fast-tests
	$(MAKE) everything-except-non-incremental

.PHONY: check-exhaustively
check-exhaustively: check-immediately
	$(MAKE) witness-tests
	$(MAKE) everything-with-excess

.PHONY: check-oneshot
check-oneshot: check-immediately
	$(MAKE) run-tests
	$(MAKE) everything-with-excess
