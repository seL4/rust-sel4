#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

PERSIST ?= 1

work_root := ../..

id := rust-sel4
image_tag := $(id)
container_name := $(id)
volume_name := $(id)

uid := $(shell id -u)
gid := $(shell id -g)

ifeq ($(PERSIST),1)
	persist_docker_run_args := --mount type=volume,src=$(volume_name),dst=/nix
endif

.PHONY: none
none:

.PHONY: build
build:
	docker build \
		--build-arg UID=$(uid) \
		--build-arg GID=$(gid) \
		-t $(image_tag) .

.PHONY: run
run: build
	docker run --privileged -d \
		--name $(container_name) \
		$(persist_docker_run_args) \
		--mount type=bind,src=$(abspath $(work_root)),dst=/work \
		--publish 8080:8080/tcp \
		--publish 8443:8443/tcp \
		$(image_tag) sleep inf

.PHONY: exec
exec:
	docker exec -it $(container_name) bash

.PHONY: show-nix-root
show-nix-root:
	docker volume inspect $(volume_name) --format='{{.Mountpoint}}'

.PHONY: rm-container
rm-container:
	for id in $$(docker ps -aq -f "name=^$(container_name)$$"); do \
		docker rm -f --volumes $$id; \
	done

.PHONY: rm-volume
rm-volume:
	for id in $$(docker volume ls -q -f "name=^$(volume_name)$$"); do \
		docker volume rm $$id; \
	done
