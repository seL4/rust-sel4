here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

top_level_dir := $(here)/..

build_subdir := docs

manifest_path := $(top_level_dir)/Cargo.toml
top_level_build_dir := $(top_level_dir)/build
build_dir := $(top_level_build_dir)/$(build_subdir)
target_dir := $(build_dir)/target

.PHONY: all
all: docs

.PHONY: clean
clean:
	rm -rf $(build_dir)

.PHONY: docs
docs:
	cargo doc \
		--locked \
		--manifest-path $(abspath $(manifest_path)) \
		--target-dir $(abspath $(target_dir)) \
		--target $(RUST_SEL4_TARGET) \
		-p sel4-placeholder
