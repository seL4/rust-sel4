root := ../../..

.PHONY: clean
clean:
	rm -rf \
		target \
		Cargo.lock \
		registry-link \
	rm -rf \
		~/.cargo/registry/*/-*

.INTERMEDIATE: registry-link-intermediate
registry-link-intermediate:
	nix-build $(root) -A pkgs.build.this.registry.staticRegistry --out-link registry-link

registry-link: registry-link-intermediate

.PHONY: shell
shell:
	nix-shell $(root) -A worlds.default.shell

.PHONY: test-build
test-build: registry-link
	RUST_TARGET_PATH=$(abspath $(root)/support/targets) \
	CARGO_REGISTRIES_SEL4_INDEX=file://$(abspath registry-link)/index \
		cargo build \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target aarch64-sel4

.PHONY: test-install
test-install: registry-link
	RUST_TARGET_PATH=$(abspath $(root)/support/targets) \
	CARGO_REGISTRIES_SEL4_INDEX=file://$(abspath registry-link)/index \
		cargo install \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target aarch64-sel4-minimal \
			--registry sel4 \
			--config .cargo/config \
			capdl-initializer
