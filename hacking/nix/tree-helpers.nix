#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

rec {
  mkLeaf = value: {
    __leaf = null;
    inherit value;
  };

  untree = lib.mapAttrs (k: v:
    if lib.isAttrs v
    then (
      if v ? __leaf
      then v.value
      else untree v
    )
    else v
  );

  mapLeaves = f: lib.mapAttrs (k: v:
    if lib.isAttrs v
    then (
      if v ? __leaf
      then mkLeaf (f v.value)
      else mapLeaves f v
    )
    else v
  );

  leaves =
    let
      f = acc: v:
        if lib.isAttrs v
        then lib.mapAttrsToList (k': f (acc ++ [k']))
        else acc
      ;
    in
      f [];
}
