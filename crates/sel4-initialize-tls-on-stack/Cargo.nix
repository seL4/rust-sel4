#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-initialize-tls-on-stack";
  dependencies = {
    inherit (versions) cfg-if;
  };
}
