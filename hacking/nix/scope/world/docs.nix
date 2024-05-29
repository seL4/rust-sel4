#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPlatform, hostPlatform, buildPackages
, runCommandCC, linkFarm, symlinkJoin, emptyDirectory
, rsync

, crateUtils
, defaultRustEnvironment
, pruneLockfile, vendorLockfile
, buildSysroot
, libclangPath
, crates

, mkSeL4RustTargetTriple
, seL4RustEnvVars

, worldConfig

, seL4Config
, seL4ConfigJSON
}:

let
  rustEnvironment = defaultRustEnvironment;
  inherit (rustEnvironment) rustToolchain;

  rootCrate = crates.meta;

  mkView = { runtime ? null, minimal ? true }:
    let
      commonFeatures = lib.optionals (!worldConfig.isMicrokit) [ "sel4-platform-info" ];
      targetTriple = mkSeL4RustTargetTriple { microkit = runtime == "sel4-microkit"; inherit minimal; };
    in {
      inherit seL4ConfigJSON;
      inherit (seL4Config) PLAT SEL4_ARCH KERNEL_MCS;
      inherit targetTriple;
      targetJSON =  "${rustEnvironment.mkTargetPath targetTriple}/${targetTriple}.json";
      inherit runtime;
      rustdoc = buildDocs {
        inherit targetTriple;
        features = commonFeatures ++ lib.optional (runtime != null) runtime;
      };
    };

  buildDocs = { targetTriple, features }:
    let
      prunedLockfile = pruneLockfile {
        inherit (rustEnvironment) rustToolchain vendoredSuperLockfile;
        rootCrates = [ rootCrate ];
      };

      vendoredLockfile = vendorLockfile {
        inherit (rustEnvironment) rustToolchain;
        lockfileContents = builtins.readFile prunedLockfile;
      };

      inherit (vendoredLockfile) lockfile;

      sysrootHost = buildSysroot {
        inherit targetTriple;
        release = false;
      };

      sysroot = sysrootHost;

      # TODO how does this work without std?
      # sysrootBuild = buildSysroot {
      #   targetTriple = buildPlatform.config;
      #   release = false;
      # };

      # sysroot = symlinkJoin {
      #   name = "sysroot";
      #   paths = [
      #     sysroot'
      #     sysroot''
      #   ];
      # };

      workspace = linkFarm "workspace" [
        { name = "Cargo.toml"; path = manifest; }
        { name = "Cargo.lock"; path = lockfile; }
        { name = "src"; path = src; }
      ];

      manifest = crateUtils.toTOMLFile "Cargo.toml" ({
        workspace.resolver = "2";
        workspace.members = [ "src/${rootCrate.name}" ];
      });

      src = crateUtils.collectReals (lib.attrValues rootCrate.closure);

      config = crateUtils.toTOMLFile "config" (crateUtils.clobber [
        (crateUtils.baseConfig {
          inherit rustEnvironment targetTriple;
        })
        vendoredLockfile.configFragment
        {
          unstable.unstable-options = true;

          target.${targetTriple}.rustflags = [
            "--sysroot" sysroot
          ];

          # TODO
          # target.${targetTriple}.rustdocflags = [
          build.rustdocflags = [
            "--sysroot" sysroot
          ];
        }
      ]);

      flags = lib.concatStringsSep " " ([
        "--offline"
        "--frozen"
        "-p" rootCrate.name
      ] ++ lib.optionals (lib.length features > 0) [
        "--features" (lib.concatStringsSep "," features)
      ] ++ [
        "--target" targetTriple
      ]);

    in
      runCommandCC "rustdoc-view" ({
        depsBuildBuild = [ buildPackages.stdenv.cc ];
        nativeBuildInputs = [ rsync rustToolchain ];
        RUST_TARGET_PATH = rustEnvironment.mkTargetPath targetTriple;
        LIBCLANG_PATH = libclangPath;
      } // seL4RustEnvVars)  ''
        target_dir=$(pwd)/target

        cargo doc \
          --config ${config} \
          --manifest-path ${workspace}/Cargo.toml \
          --target-dir=$target_dir \
          -j $NIX_BUILD_CORES \
          ${flags}

        rsync -r $target_dir/ $out/ \
          --exclude '/debug' \
          --exclude '/*/debug' \
          --exclude '/.*.json' \
          --exclude 'CACHEDIR.TAG'
      '';

in {
  inherit mkView;
}
