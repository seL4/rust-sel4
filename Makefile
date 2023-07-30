ifeq ($(K),1)
	keep_going := -k
endif

ifneq ($(J),)
	jobs := -j$(J)
endif

out := out

nix_build := nix-build $(keep_going) $(jobs)

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
	script=$$($(nix_build) -A generatedSources.update --no-out-link) && $$script
	cargo update -w

.PHONY: check-generated-sources
check-generated-sources:
	script=$$($(nix_build) -A generatedSources.check --no-out-link) && $$script
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
	./hacking/scripts/check-generic-formatting.sh

.PHONY: check-source
check-source: check-generated-sources check-fmt check-generic-formatting

try_restore_terminal := tput smam 2> /dev/null || true

.PHONY: run-tests
run-tests:
	script=$$($(nix_build) -A runTests --no-out-link) && $$script
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

.PHONY: check-fast
check-fast: check-source
	$(MAKE) witness-fast-tests
	$(MAKE) everything-fast

.PHONY: check-exhaustively
check-exhaustively: check-source
	$(MAKE) witness-tests
	$(MAKE) everything

.PHONY: check-oneshot
check-oneshot: check-source
	$(MAKE) run-tests
	$(MAKE) everything
