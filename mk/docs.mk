here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

top_level_dir := $(here)/..

build_subdir := docs

# HACK for now
target := $(RUST_BARE_METAL_TARGET)

manifest_path := $(top_level_dir)/Cargo.toml
top_level_build_dir := $(top_level_dir)/build
build_dir := $(top_level_build_dir)/$(build_subdir)

target_dir := $(build_dir)/$(target)

.PHONY: all
all: docs

.PHONY: clean
clean:
	rm -rf $(build_dir)

.PHONY: docs
docs:
	RUSTDOCFLAGS="-Z unstable-options --enable-index-page" \
		cargo doc \
			--locked \
			--manifest-path $(abspath $(manifest_path)) \
			--target-dir $(abspath $(target_dir)) \
			--target $(target) \
			-p sel4-placeholder
