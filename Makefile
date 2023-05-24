ifeq ($(K),1)
	keep_going := -k
endif

ifneq ($(J),)
	jobs := -j$(J)
endif

nix_root = hacking/nix

nix_build = nix-build $(nix_root) $(keep_going) $(jobs)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf target

.PHONY: everything
everything:
	$(nix_build) -A everything --no-out-link

.PHONY: everything-with-excess
everything-with-excess:
	$(nix_build) -A everythingWithExcess --no-out-link

.PHONY: run-automated-tests
run-automated-tests:
	script=$$($(nix_build) -A runAutomatedTests --no-out-link) && $$script
	tput smam

.PHONY: example
example:
	script=$$($(nix_build) -A example --no-out-link) && $$script

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

.PHONY: generate-target-specs
generate-target-specs:
	cargo run -p sel4-generate-target-specs -- write --target-dir support/targets/ --all

.PHONY: check-generated-sources
check-generated-sources:
	script=$$($(nix_build) -A generatedSources.check --no-out-link) && $$script

.PHONY: update-generated-sources
update-generated-sources:
	script=$$($(nix_build) -A generatedSources.update --no-out-link) && $$script
	cargo update -w
