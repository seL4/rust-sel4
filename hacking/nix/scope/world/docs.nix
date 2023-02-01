{ lib, buildPlatform, hostPlatform, buildPackages
, runCommandCC, linkFarm, symlinkJoin, emptyDirectory
, rsync

, crateUtils
, defaultRustToolchain
, defaultRustTargetInfo
, vendorLockfile, pruneLockfile
, crates
, buildSysroot

, topLevelLockfile
, vendoredTopLevelLockfile

, seL4ForUserspace
}:

let
  rustToolchain = defaultRustToolchain;
  rustTargetInfo = defaultRustTargetInfo;
  rustTargetName = rustTargetInfo.name;
  rustTargetPath = rustTargetInfo.path;

  runtimes = [
    { name = "none"; features = []; }
  ] ++ lib.optionals (hostPlatform.isAarch64 || hostPlatform.isx86_64) [
    (rec { name = "minimal-root-task-runtime"; features = [ name ]; })
  ] ++ lib.optionals hostPlatform.isAarch64 [
    (rec { name = "full-root-task-runtime"; features = [ name ]; })
  ];

  rootCrate = crates.meta-for-docs;

  # TODO try using build-std

  mk = { name, features }:
    let
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
          target.${rustTargetName}.rustflags = [
            "--sysroot" sysroot
          ];
        }
        {
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
      runCommandCC "docs-${name}" {
        depsBuildBuild = [ buildPackages.stdenv.cc ];
        nativeBuildInputs = [ rsync rustToolchain ];

        RUST_TARGET_PATH = rustTargetPath;
        LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";
        SEL4_PREFIX = seL4ForUserspace;

      } ''
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
  lib.forEach runtimes ({ name, features }@runtime: {
    inherit name;
    drv = mk runtime;
  })
