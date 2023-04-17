{ lib, stdenv, buildPlatform, hostPlatform, targetPlatform
, callPackage
, linkFarm
, treeHelpers
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

  sources = callPackage ./sources.nix {};

  seL4Arch =
    lib.head
      (map
        (x: lib.elemAt x 1)
        (lib.filter
          (x: lib.elemAt x 0)
          (with hostPlatform; [
            [ isAarch64 "aarch64" ]
            [ isAarch32 "aarch32" ]
            [ isRiscV64 "riscv64" ]
            [ isRiscV32 "riscv32" ]
            [ isx86_64 "x86_64" ]
            [ isx86_32 "ia32" ]
          ])));

  ### rust

  defaultRustToolchain = rustToolchain;

  rustTargetArchName = {
    aarch64 = "aarch64";
    aarch32 = "armv7a";
    riscv64 = "riscv64imac";
    riscv32 = "riscv32imac";
    x86_64 = "x86_64";
    ia32 = "i686";
  }."${seL4Arch}";

  defaultRustTargetInfo =
    if !hostPlatform.isNone
    then mkBuiltinRustTargetInfo hostPlatform.config
    else seL4RustTargetInfoWithConfig {};

  seL4RustTargetInfoWithConfig = { cp ? false, minimal ? false }: mkCustomRustTargetInfo "${rustTargetArchName}-sel4${lib.optionalString cp "cp"}${lib.optionalString minimal "-minimal"}";

  bareMetalRustTargetInfo = mkBuiltinRustTargetInfo {
    aarch64 = "aarch64-unknown-none";
    aarch32 = "armv7a-none-eabi"; # armv7a-none-eabihf?
    riscv64 = "riscv64imac-unknown-none-elf"; # gc?
    riscv32 = "riscv32imac-unknown-none-elf"; # gc?
    x86_64 = "x86_64-unknown-none";
    ia32 = "i686-unknown-linux-gnu";
  }."${seL4Arch}";

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
          { name = fname; path = sources.srcRoot + "/support/targets/${fname}"; }
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

  topLevelLockfile = sources.srcRoot + "/Cargo.lock";

  vendoredTopLevelLockfile = vendorLockfile { lockfile = topLevelLockfile; };

  ### upstream tools

  capdl-tool = callPackage ./capdl-tool {};

  ### misc tools

  pyoxidizer = callPackage ./pyoxidizer/pyoxidizer.nix {};
  pyoxidizerBroken = callPackage ./pyoxidizer/pyoxidizer-broken.nix {};

  ### local tools

  mkTool = rootCrate: buildCrateInLayersHere {
    inherit rootCrate;
    release = false;
  };

  sel4-inject-phdrs = mkTool crates.sel4-inject-phdrs;
  sel4-embed-debug-info = mkTool crates.sel4-embed-debug-info;
  sel4-symbolize-backtrace = mkTool crates.sel4-symbolize-backtrace;
  capdl-add-spec-to-loader = mkTool crates.capdl-add-spec-to-loader;
  sel4-simple-task-serialize-runtime-config = mkTool crates.sel4-simple-task-serialize-runtime-config;

  embedDebugInfo = callPackage ./embed-debug-info.nix {};
  injectPhdrs = callPackage ./inject-phdrs.nix {};

  ### kernel

  mkSeL4 = callPackage ./sel4 {};

  mkSeL4CorePlatform = callPackage ./sel4cp {};

  cmakeConfigHelpers = callPackage ./cmake-config-helpers.nix {};

  ### worlds

  mkWorld = unelaboratedWorldConfig: lib.makeScope newScope (callPackage ./world {} unelaboratedWorldConfig);

  worlds = (callPackage ./worlds.nix {})."${seL4Arch}";

  platUtils = callPackage ./plat-utils {};

  ### sel4test

  mkSeL4Test = callPackage ./sel4test {};

  # TODO name more configurations
  sel4test = makeOverridable' mkSeL4Test {
    rust = hostPlatform.is64bit;
  };

  ### helpers

  # Like to `lib.callPackageWith`, except without `lib.makeOverridable`.
  callWith = autoArgs: fn: args:
    let
      f = if lib.isFunction fn then fn else import fn;
      auto = builtins.intersectAttrs (lib.functionArgs f) autoArgs;
    in f (auto // args);

  # Like `lib.makeOverridable`, except it adds an orthogonal dimension of overrideablility
  # accessible at `.override'`.
  makeOverridable' = f: origArgs:
    let
      overrideWith = newArgs: origArgs // (if lib.isFunction newArgs then newArgs origArgs else newArgs);
    in f origArgs // {
      override' = newArgs: makeOverridable' f (overrideWith newArgs);
    };

})
