#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ worldConfig
, microkit
}:

microkit.genSDF {
  board = worldConfig.microkitConfig.board;
}
