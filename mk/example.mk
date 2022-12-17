here := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
root_dir := $(here)/..
build_subdir := example/$(ARCH)

include $(here)/common.mk

target_dir := $(build_dir)/target

cargo_build := \
	cargo build \
		--locked \
		--manifest-path $(abspath $(manifest_path)) \
		--target-dir $(abspath $(target_dir)) \
		--out-dir $(build_dir)

.PHONY: default
default: build

app_crate := minimal-without-runtime
app := $(build_dir)/$(app_crate).elf
app_intermediate := $(build_dir)/app.intermediate

$(app): $(app_intermediate)

.INTERMDIATE: $(app_intermediate)
$(app_intermediate):
	$(cargo_build) \
		--target $(RUST_SEL4_TARGET) \
		-p $(app_crate)

ifeq ($(ARCH),x86_64)

build_targets := $(app)

qemu_args := -initrd $(app)

else

loader_crate := loader
loader := $(build_dir)/$(loader_crate)
loader_intermediate := $(build_dir)/loader.intermediate

$(loader): $(loader_intermediate)

.INTERMDIATE: $(loader_intermediate)
$(loader_intermediate): $(app)
	SEL4_APP=$(abspath $(app)) \
		$(cargo_build) \
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
