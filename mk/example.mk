here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

top_level_dir := $(here)/..

build_subdir := example

manifest_path := $(top_level_dir)/Cargo.toml
top_level_build_dir := $(top_level_dir)/build
build_dir := $(top_level_build_dir)/$(build_subdir)
target_dir := $(build_dir)/target

sel4_target := $(RUST_SEL4_TARGET)
bare_metal_target := $(RUST_BARE_METAL_TARGET)

app_crate := minimal-without-runtime
app := $(build_dir)/$(app_crate).elf
app_intermediate := $(build_dir)/app.intermediate

loader_crate := loader
loader := $(build_dir)/$(loader_crate)
loader_intermediate := $(build_dir)/loader.intermediate

.PHONY: all
all: $(loader)

.PHONY: clean
clean:
	rm -rf $(build_dir)

$(app): $(app_intermediate)

.INTERMDIATE: $(app_intermediate)
$(app_intermediate):
	pwd
	cargo build \
		-Z unstable-options \
		--locked \
		--target $(sel4_target) \
		--target-dir $(target_dir) \
		--out-dir $(build_dir) \
		-p $(app_crate)

$(loader): $(loader_intermediate)

.INTERMDIATE: $(loader_intermediate)
$(loader_intermediate): $(app)
	SEL4_APP=$(abspath $(app)) \
		cargo build \
			-Z unstable-options \
			--locked \
			--target $(bare_metal_target) \
			--target-dir $(target_dir) \
			--out-dir $(build_dir) \
			-p $(loader_crate)

.PHONY: run
run: $(loader)
	qemu-system-aarch64 \
		-machine virt,virtualization=on \
			-cpu cortex-a57 -smp 2 -m 1024 \
			-serial mon:stdio \
			-nographic \
			-kernel $(loader)
