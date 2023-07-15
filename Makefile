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

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: check-generic-formatting
check-generic-formatting:
	./hacking/scripts/check-generic-formatting.sh

.PHONY: check-source
check-source: check-generated-sources fmt-check check-generic-formatting

.PHONY: everything
everything:
	$(nix_build) -A everything --no-out-link

.PHONY: everything-with-excess
everything-with-excess:
	$(nix_build) -A everythingWithExcess --no-out-link

.PHONY: everything-else-for-ci
everything-else-for-ci:
	$(nix_build) -A everythingElseForCI --no-out-link

try_restore_terminal := tput smam 2> /dev/null || true

.PHONY: run-automated-tests
run-automated-tests:
	script=$$($(nix_build) -A runAutomatedTests --no-out-link) && $$script
	$(try_restore_terminal)

.PHONY: witness-automated-tests
witness-automated-tests:
	$(nix_build) -A witnessAutomatedTests --no-out-link
	$(try_restore_terminal)

.PHONY: example
example:
	script=$$($(nix_build) -A $@ --no-out-link) && $$script

.PHONY: example-rpi4-b-4gb
example-rpi4-b-4gb:
	$(nix_build) -A $@ -o $(out)/$@

.PHONY: check
check: check-source
	$(MAKE) witness-automated-tests
	$(MAKE) everything

.PHONY: exhaustive-check
exhaustive-check: check-source
	$(MAKE) witness-automated-tests
	$(MAKE) everything-with-excess

.PHONY:	ci-check
ci-check: check-source
	$(MAKE) run-automated-tests
	$(MAKE) everything-else-for-ci
