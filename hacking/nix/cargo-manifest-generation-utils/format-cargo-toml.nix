#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand, rustfmtWithTOMLSupport }:

unformatted:

runCommand unformatted.name {
  nativeBuildInputs = [
    rustfmtWithTOMLSupport
  ];
} ''
  cp --no-preserve=owner,mode ${unformatted} Cargo.toml
  rustfmt --config format_cargo_toml=true Cargo.toml
  mv Cargo.toml $out
''
