{ lib, runCommand, linkFarm
, fetchurl
, jq
, defaultRustToolchain
, rustToolchain ? defaultRustToolchain
, buildPackages
}:

{ lockfileContents ? null
, lockfile ? null
, fetchGitSubmodules ? false
} @ args:

assert lockfileContents != null || lockfile != null;

let
  cratesIORegistryURL = "https://github.com/rust-lang/crates.io-index";

  lockfileContents =
    if lockfile != null
    then builtins.readFile lockfile
    else args.lockfileContents;

  lockfileValue = builtins.fromTOML lockfileContents;

  packages =
    # TODO enforce?
    # assert lockfileValue.version == 3;
    lockfileValue.package;

  remotePackages = lib.filter (lib.hasAttr "source") packages;

  parseRemotePackage = package:
    let
      parsedSource = parseSource package.source;
    in {
      inherit (package) name version;
      checksum = package.checksum or null;
      source = parsedSource;
    };

  parseSource = source:
    let
      parts = builtins.match ''([^+]*)\+(.*)'' source;
      type = lib.elemAt parts 0;
      value = lib.elemAt parts 1;
    in {
      inherit type;
      value = {
        git = parseGitSource;
        registry = parseRegistrySource;
      }.${type} value;
    };

  parseRegistrySource = registrySource: {
    registryURL = registrySource;
  };

  parseGitSource = gitSource:
    let
      parts = builtins.match ''([^?]*)(\?(rev|tag|branch)=([^#]*))?#(.*)'' gitSource;
      url = builtins.elemAt parts 0;
      refType = builtins.elemAt parts 2;
      refValue = builtins.elemAt parts 3;
      rev = builtins.elemAt parts 4;
    in {
      inherit url rev;
      ref = if refType == null then null else {
        type = refType;
        value = refValue;
      };
    };

  vendorRemotePackage = parsedRemotePackage:
    let
    in {
      git =
        assert parsedRemotePackage.checksum == null;
        vendorRemoteGitPackage {
          inherit (parsedRemotePackage) name version;
          inherit (parsedRemotePackage.source.value) url rev ref;
        };
      registry =
        assert parsedRemotePackage.checksum != null;
        assert parsedRemotePackage.source.value.registryURL == cratesIORegistryURL;
        vendorRemoteCratesIOPackage {
          inherit (parsedRemotePackage) name version checksum;
        };
    }.${parsedRemotePackage.source.type};

  vendorRemoteCratesIOPackage = { name, version, checksum }:
    let
      tarball = fetchurl {
        name = "crate-${name}-${version}.tar.gz";
        url = "https://crates.io/api/v1/crates/${name}/${version}/download";
        sha256 = checksum;
      };
      tree = runCommand "${name}-${version}" {} ''
        mkdir $out
        tar xf "${tarball}" -C $out --strip-components=1

        # Cargo is happy with partially empty metadata.
        printf '{"files": {}, "package": "${checksum}"}' > "$out/.cargo-checksum.json"
      '';
    in {
      inherit tree;
      configFragment = {};
    };

  vendorRemoteGitPackage = { name, version, url, rev, ref }:
    let
      superTree = builtins.fetchGit {
        inherit url rev;
        submodules = fetchGitSubmodules;
        allRefs = true; # HACK
      };

      # TODO
      # // (if ref != null then {
      #   ref = ref.value;
      # } else {
      #   # allRefs = true; # HACK
      # });

      tree = runCommand "${name}-${version}" {
        nativeBuildInputs = [ jq rustToolchain ];
      } ''
        tree=${superTree}

        # If the target package is in a workspace, or if it's the top-level
        # crate, we should find the crate path using `cargo metadata`.
        # Some packages do not have a Cargo.toml at the top-level,
        # but only in nested directories.
        # Only check the top-level Cargo.toml, if it actually exists
        if [[ -f $tree/Cargo.toml ]]; then
          crateCargoTOML=$(cargo metadata --format-version 1 --no-deps --manifest-path $tree/Cargo.toml | \
          jq -r '.packages[] | select(.name == "${name}") | .manifest_path')
        fi

        # If the repository is not a workspace the package might be in a subdirectory.
        if [[ -z $crateCargoTOML ]]; then
          for manifest in $(find $tree -name "Cargo.toml"); do
            echo Looking at $manifest
            crateCargoTOML=$(cargo metadata --format-version 1 --no-deps --manifest-path "$manifest" | jq -r '.packages[] | select(.name == "${name}") | .manifest_path' || :)
            if [[ ! -z $crateCargoTOML ]]; then
              break
            fi
          done

          if [[ -z $crateCargoTOML ]]; then
            >&2 echo "Cannot find path for crate '${name}-${version}' in the tree in: $tree"
            exit 1
          fi
        fi

        echo Found crate ${name} at $crateCargoTOML
        tree=$(dirname $crateCargoTOML)

        cp -prvd "$tree/" $out
        chmod u+w $out

        # Cargo is happy with empty metadata.
        printf '{"files": {}, "package": null}' > "$out/.cargo-checksum.json"
      '';
    in {
      inherit tree;
      configFragment = {
        source.${url} = {
          git = url;
          replace-with = "vendored-sources";
        } // lib.optionalAttrs (ref != null) {
          "${ref.type}" = ref.value;
        };
      };
    };

  parseAndVendorRemotePackage = remotePackage: rec {
    parsed = parseRemotePackage remotePackage;
    vendored = vendorRemotePackage parsed;
  };

  parsedAndVendoredRemotePackages = map parseAndVendorRemotePackage remotePackages;

  vendoredSourcesDirectory = linkFarm "vendored-sources" (lib.forEach parsedAndVendoredRemotePackages (p: {
    name = pathForPackage p.parsed;
    path = p.vendored.tree;
  }));

  aggregateConfigFragment =
    let
      base = {
        source.crates-io.replace-with = "vendored-sources";
        source.vendored-sources.directory = vendoredSourcesDirectory;
      };
    in
      lib.fold lib.recursiveUpdate base
        (map (p: p.vendored.configFragment) parsedAndVendoredRemotePackages);

  pathForPackage = { name, version, checksum, source }:
    let
      suffixLen = 8;
      suffix = {
        git = lib.substring 0 suffixLen source.value.rev;
        registry = lib.substring 0 suffixLen checksum;
      }.${source.type};
    in
      "${name}-${version}-${suffix}";

in {
  inherit vendoredSourcesDirectory;
  configFragment = aggregateConfigFragment;
}
