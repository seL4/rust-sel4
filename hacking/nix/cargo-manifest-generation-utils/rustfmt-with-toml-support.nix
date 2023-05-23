{ stdenv, hostPlatform
, fetchFromGitHub
, rustToolchain
, crateUtils
, vendorLockfile
}:

let
  rustTargetName = hostPlatform.config;

  src = fetchFromGitHub {
    owner = "xxchan";
    repo = "rustfmt";
    rev = "1ad83c1d48ac2f5717ea8ae398443510c95734b1";
    hash = "sha256-YJ9qNpSnEmOEb45TZcs/HwnZRWOTIXKqvW+f65MtMVE=";
  };

  cargoConfig = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    (crateUtils.linkerConfig { inherit rustToolchain rustTargetName; })
    (vendorLockfile { lockfile = "${src}/Cargo.lock"; }).configFragment
  ]);

in
stdenv.mkDerivation {
  name = "rustfmt-with-toml-support";
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
      --target ${rustTargetName} \
      --out-dir $out/bin \
      -j $NIX_BUILD_CORES
  '';
}
