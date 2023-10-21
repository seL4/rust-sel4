#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPlatform, hostPlatform, buildPackages
, runCommandCC, linkFarm, symlinkJoin, emptyDirectory
, rsync

, crateUtils
, defaultRustToolchain
, defaultRustTargetInfo
, libclangPath
, vendorLockfile, pruneLockfile
, crates
, buildSysroot
, seL4RustTargetInfoWithConfig

, topLevelLockfile
, vendoredTopLevelLockfile

, seL4RustEnvVars

, worldConfig

, seL4ConfigJSON
, seL4Config
}:

let
  rustToolchain = defaultRustToolchain;

  runtimes =
    let
      common = lib.optionals (!worldConfig.isMicrokit) [ "sel4-platform-info" ];
    in [
      { name = "none";
        features = common ++ [];
        rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = false; };
      }
    ] ++ lib.optionals (hostPlatform.isAarch64 || hostPlatform.isx86_64) [
      { name = "sel4-root-task";
        features = common ++ [ "sel4-root-task" ];
        rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = false; };
      }
    ] ++ lib.optionals (worldConfig.isMicrokit or false) [
      { name = "sel4-microkit";
        features = common ++ [ "sel4-microkit" ];
        rustTargetInfo = seL4RustTargetInfoWithConfig { microkit = true; minimal = true; };
      }
    ];

  rootCrate = crates.meta;

  mkView = { runtime ? null, minimal ? true }:
    let
      commonFeatures = lib.optionals (!worldConfig.isMicrokit) [ "sel4-platform-info" ];
      rustTargetInfo = seL4RustTargetInfoWithConfig { microkit = runtime == "sel4-microkit"; inherit minimal; };
    in {
      inherit seL4ConfigJSON;
      inherit (seL4Config) PLAT SEL4_ARCH KERNEL_MCS;
      hypervisorSupport = if seL4Config.ARCH_ARM then seL4Config.ARM_HYPERVISOR_SUPPORT else false;
      targetName = rustTargetInfo.name;
      targetJSON = "${rustTargetInfo.path}/${rustTargetInfo.name}.json";
      inherit runtime;
      rustdoc = buildDocs {
        features = commonFeatures ++ lib.optional (runtime != null) runtime;
        inherit rustTargetInfo;
      };
    };

  buildDocs = { features, rustTargetInfo }:
    let
      rustTargetName = rustTargetInfo.name;
      rustTargetPath = rustTargetInfo.path;

      lockfile = builtins.toFile "Cargo.lock" lockfileContents;
      lockfileContents = builtins.readFile lockfileDrv;
      lockfileDrv = pruneLockfile {
        superLockfile = topLevelLockfile;
        superLockfileVendoringConfig = vendoredTopLevelLockfile.configFragment;
        rootCrates = [ rootCrate ];
      };

      sysrootHost = buildSysroot {
        inherit rustTargetInfo;
        release = false;
      };

      sysroot = sysrootHost;

      # TODO how does this work without std?
      # sysrootBuild = buildSysroot {
      #   rustTargetName = buildPlatform.config;
      #   rustTargetPath = emptyDirectory;
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
        (crateUtils.baseConfig { inherit rustToolchain rustTargetName; })
        (vendorLockfile { inherit lockfileContents; }).configFragment
        {
          unstable.unstable-options = true;

          target.${rustTargetName}.rustflags = [
            "--sysroot" sysroot
          ];

          # TODO
          # target.${rustTargetName}.rustdocflags = [
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
        "--target" rustTargetName
      ]);

    in
      runCommandCC "rustdoc-view" ({
        depsBuildBuild = [ buildPackages.stdenv.cc ];
        nativeBuildInputs = [ rsync rustToolchain ];

        RUST_TARGET_PATH = rustTargetPath;
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
