#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

work_root := ../../..
here_relative := hacking/kani

id := rust-sel4-kani
image_tag := $(id)
container_name := $(id)

uid := $(shell id -u)
gid := $(shell id -g)

mount_params := type=bind,src=$(abspath $(work_root)),dst=/work

.PHONY: none
none:

.PHONY: build
build:
	docker build \
		--build-arg UID=$(uid) --build-arg GID=$(gid) \
		-t $(image_tag) .

.PHONY: runi
runi: build
	docker run --rm -it \
		--mount $(mount_params) \
		$(image_tag) bash

.PHONY: run
run: build
	docker run -d \
		--mount $(mount_params) \
		--name $(container_name) \
		$(image_tag) sleep inf

.PHONY: exec
exec:
	docker exec -it $(container_name) bash

.PHONY: rm-container
rm-container:
	for id in $$(docker ps -aq -f "name=^$(container_name)$$"); do \
		docker rm -f $$id; \
	done

.PHONY: check
check: build
	docker run --rm -it \
		--mount $(mount_params),readonly \
		$(image_tag) \
		make -C $(here_relative) check BUILD=/tmp/build
