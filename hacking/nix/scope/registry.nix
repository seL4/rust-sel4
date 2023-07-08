{ lib
, runCommand, writeText, linkFarm
, git
, cargo-index
, crateUtils
, defaultRustToolchain
, crates
, cargo-helpers
, emptyFile

, pruneLockfile
, topLevelLockfile
, vendoredTopLevelLockfile

, rustToolchain ? defaultRustToolchain
}:

let
  cratesWithRegistryDeps = lib.flip lib.mapAttrs crates (_: crate:
    let
      f = attrPath: depName: depAttrs: state: {
        value =
          if depAttrs ? path
          then depAttrs // {
            path = "../${depName}";
            version = crates.${depName}.version;
            registry = "sel4";
          }
          else depAttrs
        ;
        inherit state;
      };
      patchedManifest = crateUtils.toTOMLFile "Cargo.toml" ((crateUtils.traverseDependencies f crate.manifest null)).attrs;
    in
      # TODO symlink when resolveLinks = false
      runCommand "crate" {} ''
        cp -rL --no-preserve=mode,ownership ${crate.real} $out
        cp ${patchedManifest} $out/Cargo.toml
      ''
  );

  GIT_AUTHOR_NAME = "x";
  GIT_AUTHOR_EMAIL = "x@x";

  cratesIOIndex = builtins.fetchGit {
    url = "https://github.com/rust-lang/crates.io-index.git";
    rev = "4dd500cfd31cc7ac2afb68f9a8263d072ac0fc42";
  };

  cratesIOIndexBundle = runCommand "crates-io-index.bundle" {
    nativeBuildInputs = [
      git
    ];
  } ''
    cp -r --no-preserve=mode,ownership ${cratesIOIndex} x
    cd x
    echo '{ "dl": "file:///var/empty/crates" }' > config.json
    git init -b main
    git add --all
    git -c user.name=${GIT_AUTHOR_NAME} -c user.email=${GIT_AUTHOR_EMAIL} commit -m x --quiet
    git bundle create $out main
  '';

  cratesIOIndexRepo = runCommand "crates-io-index.git" {
    nativeBuildInputs = [
      git
    ];
  } ''
    git clone --bare ${cratesIOIndexBundle} -b main $out
    rm -r \
      $out/hooks \
      $out/description
  '';

  localCratesIOCargoConfig = {
    source.crates-io.replace-with = "local-crates-io";
    source.local-crates-io.registry = "file:///${cratesIOIndexRepo}";
  };

  cargoCachePopulatedWithCratesIOIndex =
    let
      config = crateUtils.toTOMLFile "x" localCratesIOCargoConfig;
      workspace = linkFarm "x" [
        { name = "Cargo.toml";
          path = crateUtils.toTOMLFile "x" {
            package = {
              name = "x";
              version = "0.0.0";
            };
            dependencies = {
              cfg-if = "*";
            };
          };
        }
        {
          name = "src/lib.rs";
          path = emptyFile;
        }
      ];
    in
      runCommand "cargo-home" {
        nativeBuildInputs = [
          rustToolchain
        ];
      } ''
        export CARGO_HOME=$out

        cp -r ${workspace}/* .

        cargo generate-lockfile \
          --config ${config}
      '';

  orderCrates = theseCrates: (lib.toposort (a: b: lib.hasAttr a.name b.closure) theseCrates).result;

  orderedCrateNames = map (crate: crate.name) (orderCrates (lib.attrValues crates));

  staticRegistry =
    let
      cargoConfig = crateUtils.toTOMLFile "x" localCratesIOCargoConfig;

      lockfile = builtins.toFile "Cargo.lock" lockfileContents;
      lockfileContents = builtins.readFile lockfileDrv;
      lockfileDrv = pruneLockfile {
        superLockfile = topLevelLockfile;
        superLockfileVendoringConfig = vendoredTopLevelLockfile.configFragment;
        rootCrates = lib.attrValues crates;
      };

      manifest = crateUtils.toTOMLFile "Cargo.toml" {
        workspace.resolver = "2";
        workspace.members = lib.forEach orderedCrateNames (crateName: "src/${crateName}");
      };

      src = linkFarm "crates" (lib.flip lib.mapAttrsToList cratesWithRegistryDeps (name: path: {
        inherit name path;
      }));

      workspace = linkFarm "workspace" [
        { name = "Cargo.toml"; path = manifest; }
        { name = "Cargo.lock"; path = lockfile; }
        { name = "src"; path = src; }
        { name = ".cargo/config"; path = cargoConfig; }
      ];

      localCratesIOIndexURL = "file:///${cratesIOIndexRepo}";
      dummyIndexURL = "file:///var/empty/sel4-dummy/index";
    in
      runCommand "index" {
        nativeBuildInputs = [
          git
          rustToolchain
          cargo-index
          cargo-helpers
        ];
        inherit GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL;
      } ''
        index=$out/index
        crates=$out/crates
        mkdir -p $crates

        target_dir=$(pwd)/target

        export CARGO_HOME=$(pwd)/cargo-home

        cp -r --no-preserve=ownership,mode ${cargoCachePopulatedWithCratesIOIndex} $CARGO_HOME

        export CARGO_REGISTRIES_SEL4_INDEX=file://$index

        cargo index init --index $index --dl file://$crates/'{crate}-{version}.crate'

        for p in ${lib.concatStringsSep " " orderedCrateNames}; do
          cargo package \
            --config ${cargoConfig} \
            --manifest-path ${workspace}/Cargo.toml \
            --target-dir $target_dir \
            --locked \
            --no-verify \
            -p $p

          crate_archive=$target_dir/package/$p-*.crate
          tar -xzf $crate_archive
          rm $crate_archive

          p_with_version=$(echo $p-*)

          for f in Cargo.toml Cargo.lock; do
            if [ -f $p_with_version/$f ]; then
              sed -i "s|file://$index|${dummyIndexURL}|" $p_with_version/$f
            fi
          done

          tar -czf $p_with_version.crate $p_with_version
          rm -r $p_with_version
          mv $p_with_version.crate $crates

          cargo index add \
            --index $index \
            --index-url "${dummyIndexURL}" \
            --crate $crates/$p_with_version.crate
        done

        for p in ${lib.concatStringsSep " " orderedCrateNames}; do
          path=$(cargo-helpers make-dep-path $p)
          sed -i "s|file://$index|${dummyIndexURL}|" $index/$path
        done
      '';

  registryIndex = "${staticRegistry}/index";
  registryCrates = "${staticRegistry}/crates";

in rec {
  inherit cratesIOIndex cratesIOIndexBundle cratesIOIndexRepo cargoCachePopulatedWithCratesIOIndex;
  inherit cratesWithRegistryDeps;
  inherit staticRegistry registryIndex registryCrates;
}
