here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
root_dir := $(here)/..
build_subdir := docs

include $(here)/common.mk

# HACK for now
target := $(RUST_BARE_METAL_TARGET)

target_dir := $(build_dir)/$(target)

crate_args := -p sel4

.PHONY: all
all: docs

.PHONY: clean
clean:
	rm -rf $(build_dir)

cargo_invocation = \
	RUSTDOCFLAGS="-Z unstable-options --enable-index-page" \
		cargo $(1) \
			--locked \
			--manifest-path $(abspath $(manifest_path)) \
			--target-dir $(abspath $(target_dir)) \
			--target $(target) \
			$(crate_args)

.PHONY: docs
docs:
	$(call cargo_invocation,build)
	$(call cargo_invocation,doc)
