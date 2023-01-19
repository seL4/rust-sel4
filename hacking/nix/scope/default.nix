{ lib, stdenv, buildPlatform, hostPlatform, targetPlatform
, callPackage
, linkFarm
}:

let
  superCallPackage = callPackage;
in

self:

let
  callPackage = self.callPackage;
in

let
  fenixRev = "a9a262cbec1f1c3f771071fd1a246ee05811f5a1";
  fenixSource = fetchTarball "https://github.com/nix-community/fenix/archive/${fenixRev}.tar.gz";
  fenix = import fenixSource {};

  rustToolchainParams = {
    channel = "nightly";
    date = "2022-11-11";
    sha256 = "sha256-rL0Y1pPrK8RS2UbYHpFXJDMls4ZQYDWK5MbUscwUxbI=";
  };

  mkRustToolchain = target: fenix.targets.${target}.toolchainOf rustToolchainParams;

  # TODO
  # rustToolchain = fenix.combine ([
  #   (mkRustToolchain hostPlatform.config).completeToolchain
  # ] ++ lib.optionals (hostPlatform.config != targetPlatform.config && !targetPlatform.isNone) [
  #   (mkRustToolchain targetPlatform.config).rust-std
  # ]);

  rustToolchain = (mkRustToolchain buildPlatform.config).completeToolchain;

in

superCallPackage ../rust-utils {} self //

(with self; {

  srcRoot = ../../..;

  topLevelLockfile = srcRoot + "/Cargo.lock";

  defaultRustToolchain = rustToolchain;

  defaultRustTargetName =
    if !hostPlatform.isNone
    then hostPlatform.config
    else {
      aarch64 = "aarch64-unknown-sel4";
      x86_64 = "x86_64-unknown-sel4";
    }."${hostPlatform.parsed.cpu.name}";

  bareMetalRustTargetName =
    if !hostPlatform.isNone
    then hostPlatform.config
    else {
      aarch64 = "aarch64-unknown-none";
    }."${hostPlatform.parsed.cpu.name}";

  defaultRustTargetPath =
    if !hostPlatform.isNone
    then null
    else
      let
        fname = "${defaultRustTargetName}.json";
      in
        linkFarm "targets" [
          { name = fname; path = srcRoot + "/support/targets/${fname}"; }
        ];

  chooseLinkerForRustTarget = { rustToolchain, rustTargetName, platform }:
    if platform.isNone
    then "${rustToolchain}/lib/rustlib/${buildPlatform.config}/bin/rust-lld"
    else null;

  # chooseLinkerForRustTarget = { rustToolchain, rustTargetName, platform }:
  #   if platform.isNone
  #   then "${stdenv.cc.targetPrefix}ld"
  #   else null;

  crates = callPackage ./crates.nix {};

  buildCrateInLayersHere = buildCrateInLayers {
    superLockfile = topLevelLockfile;
  };

  mkTool = rootCrate: buildCrateInLayersHere {
    inherit rootCrate;
    release = false;
  };

  sel4-inject-phdrs = mkTool crates.sel4-inject-phdrs;
  sel4-symbolize-backtrace = mkTool crates.sel4-symbolize-backtrace;

  injectPhdrs = callPackage ./inject-phdrs.nix {};

  configHelpers = callPackage ./config-helpers.nix {};

  elaborateWorldConfig =
    { kernelConfig, loaderConfig
    , mkQemuCmd ? null
    , qemuCmdRequiresLoader ? true
    }:
    { inherit
        kernelConfig loaderConfig
        mkQemuCmd
        qemuCmdRequiresLoader
      ;
    };

  mkWorld = worldConfig: lib.makeScope newScope (callPackage ./world {} (elaborateWorldConfig worldConfig));

  worlds = (callPackage ./worlds.nix {})."${hostPlatform.parsed.cpu.name}";

})
