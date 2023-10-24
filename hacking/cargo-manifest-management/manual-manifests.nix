#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

let

  relativeToWorkspaceRoot = relativePath: "${toString ../..}/${relativePath}";

in {
  # "virtio-drivers" = relativeToWorkspaceRoot "tmp/virtio-drivers";
  # "mbedtls" = relativeToWorkspaceRoot "tmp/rust-mbedtls/mbedtls";
  # "mbedtls-sys-auto" = relativeToWorkspaceRoot "tmp/rust-mbedtls/mbedtls-sys";
  # "mbedtls-platform-support" = relativeToWorkspaceRoot "tmp/rust-mbedtls/mbedtls-platform-support";
  # "embedded-fat" = relativeToWorkspaceRoot "tmp/rust-embedded-fat";
  # "volatile" = relativeToWorkspaceRoot "tmp/volatile";
}
