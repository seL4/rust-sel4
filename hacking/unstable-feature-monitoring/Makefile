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
	./enumerate-used-unstable-features.sh

.PHONY: crates-using
crates-using:
	./enumerate-crates-using-unstable-features.sh

.PHONY: wishlist
wishlist:
	cargo build --manifest-path wishlist/Cargo.toml
