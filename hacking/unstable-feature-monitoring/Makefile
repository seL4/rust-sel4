#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf wishlist/target

.PHONY: used
used:
	./enumerate-unstable-features-used.sh

.PHONY: wishlist
wishlist:
	cargo build --manifest-path wishlist/Cargo.toml
