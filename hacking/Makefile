ifeq ($(K),1)
	keep_going := -k
endif

ifneq ($(J),)
	jobs := -j$(J)
endif

root = ..

nix_build = nix-build $(keep_going) $(jobs)

.PHONY: none
none:

.PHONY: everything
everything:
	$(nix_build) -A everything --no-out-link

.PHONY: run-automated-tests
run-automated-tests:
	script=$$($(nix_build) -A runAutomatedTests --no-out-link) && $$script

.PHONY: example
example:
	script=$$($(nix_build) -A example --no-out-link) && $$script

.PHONY: fmt
fmt:
	cd $(root) && cargo fmt --all

.PHONY: update-lockfile
update-lockfile:
	cd $(root) && cargo update -w

