architectures := \
	aarch64 \
	riscv64 \
	x86_64

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf target build

.PHONY: example
example:
	nix-shell --pure -A worlds.aarch64.default.shell --run "make -f mk/example.mk run"

mk_docs = nix-shell --pure -A worlds.$(1).default.shell --run "make -f mk/docs.mk"

.PHONY: docs
docs:
	$(foreach arch,$(architectures),$(call mk_docs,$(arch));)
