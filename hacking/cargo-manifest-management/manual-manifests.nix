#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

let

  relativeToWorkspaceRoot = relativePath: "${toString ../..}/${relativePath}";

  relativeToTmpSrc = relativePath: relativeToWorkspaceRoot "tmp/src/${relativePath}";

in {
  # ring = relativeToTmpSrc "ring";
  # rustls = relativeToTmpSrc "rustls/rustls";
  # lock_api = relativeToTmpSrc "parking_lot/lock_api";
}
