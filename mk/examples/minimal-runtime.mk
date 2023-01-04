CRATE ?= minimal-runtime-with-state

here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
root_dir := $(here)/../..
build_subdir := examples/minimal-runtime/$(CRATE)/$(ARCH)

include $(here)/../common.mk

target_dir := $(build_dir)/target

cargo_build := \
	cargo build \
		--locked \
		--manifest-path $(abspath $(manifest_path)) \
		--target-dir $(abspath $(target_dir)) \
		--out-dir $(build_dir)

cargo_build_std_args := \
	-Z build-std=core,alloc,compiler_builtins \
	-Z build-std-features=compiler-builtins-mem

.PHONY: default
default: build

app_crate := $(CRATE)
app := $(build_dir)/$(app_crate).elf
app_intermediate := $(build_dir)/$(app_crate).intermediate

$(app): $(app_intermediate)

.INTERMDIATE: $(app_intermediate)
$(app_intermediate):
	$(cargo_build) \
		$(cargo_build_std_args) \
		--target $(RUST_SEL4_TARGET) \
		-p $(app_crate)

ifeq ($(ARCH),x86_64)

build_targets := $(app)

qemu_args := -initrd $(app)

else

loader_crate := loader
loader := $(build_dir)/$(loader_crate)
loader_intermediate := $(build_dir)/$(loader_crate).intermediate

$(loader): $(loader_intermediate)

.INTERMDIATE: $(loader_intermediate)
$(loader_intermediate): $(app)
	SEL4_APP=$(abspath $(app)) \
		$(cargo_build) \
			$(cargo_build_std_args) \
			--target $(RUST_BARE_METAL_TARGET) \
			-p $(loader_crate)

build_targets := $(loader)

qemu_args := -kernel $(loader)

endif

.PHONY: build
build: $(build_targets)

.PHONY: run
run: build
	$(QEMU_CMD) $(qemu_args)

.PHONY: clean
clean:
	rm -rf $(build_dir)
