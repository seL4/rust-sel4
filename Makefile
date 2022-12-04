.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf target build

.PHONY: example
example:
	nix-shell --pure -A worlds.aarch64.default.shell --run "$(MAKE) -f mk/example.mk run"
