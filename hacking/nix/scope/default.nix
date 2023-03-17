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
    date = "2023-01-26";
    sha256 = "sha256-UdHZxhV3QQ+1ZdATle9/dl3zheEHw6nz14Aq4gMHj7Y=";
    # TODO
    # loader crashes when profile.{}.debug = "2"
    # date = "2023-01-27";
    # sha256 = "sha256-JHs8yGthsweFSeamjAt2wzPh4ZeR4dsmHb9tcfT9k5A=";
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

  vendoredTopLevelLockfile = vendorLockfile { lockfile = topLevelLockfile; };

  defaultRustToolchain = rustToolchain;

  rustTargetArchName = {
    aarch64 = "aarch64";
    riscv64 = "riscv64imac";
    x86_64 = "x86_64";
  }."${hostPlatform.parsed.cpu.name}";

  defaultRustTargetInfo =
    if !hostPlatform.isNone
    then mkBuiltinRustTargetInfo hostPlatform.config
    else seL4RustTargetInfoWithConfig {};

  seL4RustTargetInfoWithConfig = { cp ? false, minimal ? false }: mkCustomRustTargetInfo "${rustTargetArchName}-sel4${lib.optionalString cp "cp"}${lib.optionalString minimal "-minimal"}";

  bareMetalRustTargetInfo = mkBuiltinRustTargetInfo {
    aarch64 = "aarch64-unknown-none";
    riscv64 = "riscv64imac-unknown-none-elf";
    x86_64 = "x86_64-unknown-none";
  }."${hostPlatform.parsed.cpu.name}";

  mkBuiltinRustTargetInfo = name: {
    inherit name;
    path = null;
  };

  mkCustomRustTargetInfo = name: {
    inherit name;
    path =
      let
        fname = "${name}.json";
      in
        linkFarm "targets" [
          { name = fname; path = srcRoot + "/support/targets/${fname}"; }
        ];
  };

  chooseLinkerForRustTarget = { rustToolchain, rustTargetName, platform }:
    if platform.isNone
    then "${rustToolchain}/lib/rustlib/${buildPlatform.config}/bin/rust-lld"
    else null;

  crates = callPackage ./crates.nix {};

  buildCrateInLayersHere = buildCrateInLayers {
    # TODO pass vendored lockfile instead
    superLockfile = topLevelLockfile;
  };

  mkTool = rootCrate: buildCrateInLayersHere {
    inherit rootCrate;
    release = false;
  };

  sel4-inject-phdrs = mkTool crates.sel4-inject-phdrs;
  sel4-embed-debug-info = mkTool crates.sel4-embed-debug-info;
  sel4-symbolize-backtrace = mkTool crates.sel4-symbolize-backtrace;

  injectPhdrs = callPackage ./inject-phdrs.nix {};

  embedDebugInfo = callPackage ./embed-debug-info.nix {};

  configHelpers = callPackage ./config-helpers.nix {};

  elaborateWorldConfig =
    { kernelConfig, loaderConfig
    , mkQemuCmd ? null
    , qemuCmdRequiresLoader ? true
    , forCP ? false
    }:
    { inherit
        kernelConfig loaderConfig
        mkQemuCmd
        qemuCmdRequiresLoader
        forCP
      ;
    };

  mkWorld = worldConfig: lib.makeScope newScope (callPackage ./world {} (elaborateWorldConfig worldConfig));

  worlds = (callPackage ./worlds.nix {})."${hostPlatform.parsed.cpu.name}";

  sel4cp = callPackage ./sel4cp {};

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

  sel4test = callPackage ./sel4test {};

  sources = callPackage ./sources.nix {};

  capdl-tool = callPackage ./capdl-tool {};

})
