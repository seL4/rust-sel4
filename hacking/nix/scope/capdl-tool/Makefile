#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

top_level := ../../../..
capdl := $(top_level)/../x/capdl/capDL-tool
cabal2nix := $(shell nix-build $(top_level) -A pkgs.build.cabal2nix --no-out-link)/bin/cabal2nix

all := \
	capDL-tool.nix

.PHONY: all $(all)
all: $(all)

capDL-tool.nix:
	$(cabal2nix) file://$(abspath $(capdl)) > $@
