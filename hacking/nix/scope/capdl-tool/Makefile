#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

top_level := ../../../..
capdl := $(top_level)/../x/capdl/capDL-tool
cabal2nix := $(shell nix-build $(top_level) -A pkgs.build.cabal2nix --no-out-link)/bin/cabal2nix

all := \
	capDL-tool.nix \
	base-compat-0-12-2.nix \
	base-compat-batteries-0-12-2.nix \
	network-3.1.4.0.nix \
	MissingH-1.5.0.1.nix

.PHONY: all $(all)
all: $(all)

capDL-tool.nix:
	$(cabal2nix) file://$(abspath $(capdl)) > $@

base-compat-0-12-2.nix:
	$(cabal2nix) cabal://base-compat-0.12.2 > $@

base-compat-batteries-0-12-2.nix:
	$(cabal2nix) cabal://base-compat-batteries-0.12.2 > $@

network-3.1.4.0.nix:
	$(cabal2nix) cabal://network-3.1.4.0 > $@

MissingH-1.5.0.1.nix:
	$(cabal2nix) cabal://MissingH-1.5.0.1 > $@
