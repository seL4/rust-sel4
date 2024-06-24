#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, emptyDirectory, linkFarm, writeText }:

{
  cdl = writeText "x.cdl" ''
    arch aarch64

    objects {
      foo = notification
    }
    caps {}
  '';
  fill = emptyDirectory;
}
