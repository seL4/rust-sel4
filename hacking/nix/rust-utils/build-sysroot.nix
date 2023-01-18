{ lib, stdenv, buildPlatform, hostPlatform, buildPackages
, runCommand, linkFarm
, vendorLockfile, crateUtils, symlinkToRegularFile
, defaultRustToolchain, defaultRustTargetPath, defaultRustTargetName
, rustToolchain ? defaultRustToolchain
}:

let
  sysrootLockfile = symlinkToRegularFile "Cargo.lock" "${rustToolchain}/lib/rustlib/src/rust/Cargo.lock";

  # NOTE
  # There is one thunk per package set.
  # Consolidating further would improve eval perf.
  vendoredCrates = vendorLockfile { lockfile = sysrootLockfile; };
in

{ release ? true
, extraManifest ? {}
, extraConfig ? {}
, rustTargetName ? defaultRustTargetName
, rustTargetPath ? defaultRustTargetPath
}:

let
  workspace = linkFarm "workspace" [
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
  ];

  package = {
    name = "dummy";
    version = "0.0.0";
  };

  manifest = crateUtils.toTOMLFile "Cargo.toml" (crateUtils.clobber [
    {
      inherit package;
      lib.path = crateUtils.dummyLibInSrc;
    }
    extraManifest
  ]);

  lockfile = crateUtils.toTOMLFile "Cargo.lock" {
    package = [
      package
    ];
  };

  config = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    # baseConfig # TODO will trigger rebuild
    {
      target = {
        "${rustTargetName}" = {
          rustflags = [
            # "-C" "force-unwind-tables=yes" # TODO compare with "requires-uwtable" in target.json
            "-C" "embed-bitcode=yes"
            "--sysroot" "/dev/null"
          ];
        };
      };
    }
    vendoredCrates.configFragment
    extraConfig
  ]);

in
runCommand "sysroot" {
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  nativeBuildInputs = [ rustToolchain ];

  RUST_TARGET_PATH = rustTargetPath;
} ''
  cargo build \
    -Z unstable-options \
    --offline \
    --frozen \
    --config ${config} \
    ${lib.optionalString release "--release"} \
    --target ${rustTargetName} \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    --manifest-path ${workspace}/Cargo.toml \
    --target-dir $(pwd)/target

  d=$out/lib/rustlib/${rustTargetName}/lib
  mkdir -p $d
  mv target/${rustTargetName}/${if release then "release" else "debug"}/deps/* $d
''
