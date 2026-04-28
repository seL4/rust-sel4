#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

workspaces_dir := workspaces

private_workspaces_dir := $(workspaces_dir)/private

ws := .code-workspace

.PHONY: none
none:

.PHONY: default-excludes
default-excludes:
	cargo run -p create-vscode-workspace -- \
		-e sel4-config-data \
		-e sel4-test-harness \
		-e tests-root-task-dafny-core \
		-e lionsos-sys \
		-e sddf-sys \
		-e sel4-newlib \
		--just-dump-excludes \
		-o $(default_excludes)

.PHONY: http-server
http-server:
	cargo run -p create-vscode-workspace -- \
		--config .cargo/gen/target/aarch64-sel4-microkit.toml \
		--config .cargo/gen/world/aarch64.microkitDefault.toml \
		-i microkit-http-server-example-server \
		-o $(private_workspaces_dir)/ukit$(ws)

.PHONY: all-sel4
all-sel4:
	cargo run -p create-vscode-workspace -- \
		--config .cargo/gen/target/aarch64-sel4-microkit.toml \
		--config .cargo/gen/world/aarch64.microkitDefault.toml \
		--include-dependents sel4 \
		-e banscii-serial-driver \
		-e tests-root-task-dafny-core \
		-e sel4-root-task-with-std \
		-e sel4-test-harness \
		-e sel4-simple-task-runtime \
		-e sel4-simple-task-rpc \
		-e spawn-task \
		-e spawn-thread \
		-o $(private_workspaces_dir)/all-sel4$(ws)
