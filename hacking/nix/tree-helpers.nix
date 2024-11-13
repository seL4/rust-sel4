#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

let
  mapLeafNodes = f: lib.mapAttrs (k: v:
    assert lib.isAttrs v;
    if v ? __leaf
    then f v
    else mapLeafNodes f v
  );

in rec {
  mkLeaf = value: {
    __leaf = null;
    inherit value;
  };

  untree = mapLeafNodes (leafNode: leafNode.value);

  mapLeaves = f: mapLeafNodes (leafNode: mkLeaf (f leafNode.value));
}
