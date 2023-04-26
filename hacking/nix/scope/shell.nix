{ lib, stdenv, buildPackages
, mkShell
, defaultRustToolchain
, pkgconfig, openssl
}:

mkShell {
  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    pkgconfig
    defaultRustToolchain
  ];

  buildInputs = [
    openssl
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];
}
