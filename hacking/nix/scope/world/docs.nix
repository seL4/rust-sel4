{ lib, buildPlatform, hostPlatform, buildPackages
, runCommandCC, linkFarm, symlinkJoin, emptyDirectory
, rsync

, crateUtils
, defaultRustToolchain
, defaultRustTargetInfo
, vendorLockfile, pruneLockfile
, crates
, buildSysroot
, seL4RustTargetInfoWithConfig

, topLevelLockfile
, vendoredTopLevelLockfile

, seL4RustEnvVars

, worldConfig
}:

let
  rustToolchain = defaultRustToolchain;

  runtimes = [
    { name = "none";
      features = [];
      rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = false; };
    }
  ] ++ lib.optionals (hostPlatform.isAarch64 || hostPlatform.isx86_64) [
    { name = "root-task";
      features = [ "root-task" ] ++ lib.optionals (!worldConfig.isCorePlatform) [ "sel4-platform-info" ];
      rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = false; };
    }
  ] ++ lib.optionals (worldConfig.isCorePlatform or false) [
    { name = "sel4cp";
      features = [ "sel4cp" ];
      rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; cp = true; };
    }
  ];

  rootCrate = crates.meta;

  # TODO try using build-std

  mk = { name, features, rustTargetInfo }:
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
      runCommandCC "docs-${name}" ({
        depsBuildBuild = [ buildPackages.stdenv.cc ];
        nativeBuildInputs = [ rsync rustToolchain ];

        RUST_TARGET_PATH = rustTargetPath;
        LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";
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

in
  lib.forEach runtimes ({ name, rustTargetInfo, ... }@runtime: {
    inherit name rustTargetInfo;
    drv = mk runtime;
  })
