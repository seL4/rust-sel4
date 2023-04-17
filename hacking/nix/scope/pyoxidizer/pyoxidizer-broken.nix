{ lib, stdenv, hostPlatform
, fetchFromGitHub
, defaultRustToolchain
, crateUtils
, vendorLockfile
}:

let
  rustTargetName = hostPlatform.config;
  rustToolchain = defaultRustToolchain;

  src = fetchFromGitHub {
    owner = "indygreg";
    repo = "PyOxidizer";
    rev = "36e50354c97275c7d9f47b3c38a8c216a08771f5";
    hash = "sha256-/nMEGxkL8HKXN/9AvSx9nda8EljarLwZhqifddKTvHI=";
  };

  cargoConfig = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    (crateUtils.linkerConfig { inherit rustToolchain rustTargetName; })
    (vendorLockfile { lockfile = "${src}/Cargo.lock"; }).configFragment
  ]);

in
stdenv.mkDerivation {
  name = "pyoxidizer";
  inherit src;

  nativeBuildInputs = [
    rustToolchain
  ];

  dontConfigure = true;
  dontInstall = true;

  buildPhase = ''
    cargo build \
      -Z unstable-options \
      --offline --frozen \
      --config ${cargoConfig} \
      --release \
      --out-dir $out/bin \
      -j $NIX_BUILD_CORES
  '';
}
