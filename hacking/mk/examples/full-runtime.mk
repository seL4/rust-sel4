CRATE ?= full-runtime

here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
root_dir := $(here)/../../..
build_subdir := examples/full-runtime/$(CRATE)/$(ARCH)

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

inject_phdrs_crate := sel4-inject-phdrs
inject_phdrs := $(build_dir)/$(inject_phdrs_crate)
inject_phdrs_intermediate := $(build_dir)/$(inject_phdrs_crate).intermediate

$(inject_phdrs): $(inject_phdrs_intermediate)

.INTERMDIATE: $(inject_phdrs_intermediate)
$(inject_phdrs_intermediate):
	$(cargo_build) \
		-p $(inject_phdrs_crate)

app_crate := $(CRATE)
app := $(build_dir)/$(app_crate).elf
app_intermediate := $(build_dir)/$(app_crate).intermediate
app_with_phdrs := $(build_dir)/$(app_crate)-with-phdrs.elf

$(app): $(app_intermediate)

.INTERMDIATE: $(app_intermediate)
$(app_intermediate):
	$(cargo_build) \
		$(cargo_build_std_args) \
		--target $(RUST_SEL4_TARGET) \
		-p $(app_crate)

$(app_with_phdrs): $(app) $(inject_phdrs)
	$(inject_phdrs) $< -o $@

ifeq ($(ARCH),x86_64)

build_targets := $(app_with_phdrs)

qemu_args := -initrd $(app_with_phdrs)

else

loader_crate := loader
loader := $(build_dir)/$(loader_crate)
loader_intermediate := $(build_dir)/$(loader_crate).intermediate

$(loader): $(loader_intermediate)

.INTERMDIATE: $(loader_intermediate)
$(loader_intermediate): $(app_with_phdrs)
	SEL4_APP=$(abspath $(app_with_phdrs)) \
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
