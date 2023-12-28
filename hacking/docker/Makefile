#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

STATEFUL ?= $(if $(findstring Linux,$(shell uname -s)),0,1)

work_root := ../..

id := rust-sel4
label := $(id)
image_tag := $(id)
container_name := $(id)
volume_name := $(id)-nix-root

uid := $(shell id -u)
gid := $(shell id -g)

ifneq ($(STATEFUL),1)
	statefulness_run_prerequisites := initialize-volume
	statefulness_docker_run_args := --mount type=volume,src=$(volume_name),dst=/nix
endif

.PHONY: none
none:

.PHONY: build
build:
	docker build \
		--build-arg UID=$(uid) --build-arg GID=$(gid) --build-arg STATEFUL=$(STATEFUL) \
		--label $(label) -t $(image_tag) .

.PHONY: initialize-volume
initialize-volume: build
	if [ -z "$$(docker volume ls -q -f "name=^$(volume_name)$$")" ]; then \
		docker volume create --label $(label) $(volume_name) && \
		docker run --privileged --rm --label $(label) -w /tmp \
			$(statefulness_docker_run_args) \
			$(image_tag) flock /nix/.installed.lock bash setup.sh; \
	fi

.PHONY: run
run: build $(statefulness_run_prerequisites)
	docker run --privileged -d --name $(container_name) --label $(label) \
		$(statefulness_docker_run_args) \
		--mount type=bind,src=$(abspath $(work_root)),dst=/work \
		--publish 8080:8080/tcp \
		--publish 8443:8443/tcp \
		$(image_tag) sleep inf

.PHONY: exec
exec:
	docker exec -it $(container_name) bash

.PHONY: show-nix-root
show-nix-root:
	docker inspect $(volume_name) --format='{{.Mountpoint}}'

.PHONY: rm-container
rm-container:
	for id in $$(docker ps -aq -f "name=^$(container_name)$$"); do \
		docker rm -f $$id; \
	done

.PHONY: rm-volume
rm-volume:
	for volume in $$(docker volume ls -q -f "name=^$(volume_name)$$"); do \
		docker volume rm $$volume; \
	done
