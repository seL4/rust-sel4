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

.PHONY: everything
everything:
	$(nix_build) -A everything --no-out-link

.PHONY: everything-with-excess
everything-with-excess:
	$(nix_build) -A everythingWithExcess --no-out-link

.PHONY: run-automated-tests
run-automated-tests:
	script=$$($(nix_build) -A runAutomatedTests --no-out-link) && $$script
	tput smam || true

.PHONY: witness-automated-tests
witness-automated-tests:
	$(nix_build) -A witnessAutomatedTests --no-out-link
	tput smam || true

.PHONY: example
example:
	script=$$($(nix_build) -A $@ --no-out-link) && $$script

.PHONY: example-rpi4-b-4gb
example-rpi4-b-4gb:
	$(nix_build) -A $@ -o $(out)/$@

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: update-lockfile
update-lockfile:
	cargo update -w

.PHONY: check-lockfile
check-lockfile:
	cargo update -w --locked

rustc_target_spec_dir := support/targets

.PHONY: generate-target-specs
generate-target-specs:
	rm -f $(rustc_target_spec_dir)/*.json && \
		cargo run -p sel4-generate-target-specs -- write --target-dir $(rustc_target_spec_dir) --all

.PHONY: check-generated-sources
check-generated-sources:
	script=$$($(nix_build) -A generatedSources.check --no-out-link) && $$script
	cargo update -w --locked

.PHONY: update-generated-sources
update-generated-sources:
	script=$$($(nix_build) -A generatedSources.update --no-out-link) && $$script
	cargo update -w
