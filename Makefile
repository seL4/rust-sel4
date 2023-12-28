#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

ifeq ($(K),1)
	keep_going_args := -k
endif

ifneq ($(J),)
	jobs_args := -j$(J)
else
	jobs_args :=
endif

ifneq ($(CORES),)
	cores_args := --cores $(CORES)
else
	cores_args :=
endif

out := out

nix_args := $(keep_going_args) $(jobs_args) $(cores_args)

nix_build := nix-build $(nix_args)

nix_shell := nix-shell $(nix_args)

run_in_nix_shell := $(nix_shell) --run

ifeq ($(IN_NIX_SHELL_FOR_MAKEFILE),)
	# TODO
	# Should this use --pure? One consideration is that 'make shell' below
	# doesn't, and consistency might be more important here.
	run_in_nix_shell := $(nix_shell) -A shellForMakefile --pure --run
else
	run_in_nix_shell := $(SHELL) -c
endif

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(out) target

$(out):
	mkdir -p $@

.PHONY: shell
shell:
	$(nix_shell) -A shellForHacking

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
	$(run_in_nix_shell) "sh hacking/scripts/check-generic-formatting.sh"

.PHONY: check-source
check-source: check-generated-sources check-fmt check-generic-formatting

.PHONY: check-licenses
check-licenses:
	$(run_in_nix_shell) "reuse lint"

.PHONY: check-dependencies
check-dependencies:
	lockfile=$$($(nix_build) -A pkgs.build.this.publicCratesCargoLock --no-out-link) && \
		$(run_in_nix_shell) "cargo-audit audit -f $$lockfile"

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
