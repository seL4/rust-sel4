#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

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
, extraMkCrateTarballURLFns ? {}
} @ args:

assert lockfileContents != null || lockfile != null;

let
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

  remotePackagesBySource = lib.foldAttrs (x: xs: [x] ++ xs) [] (lib.forEach remotePackages (package: {
    "${package.source}" = builtins.removeAttrs package [ "source" ];
  }));

  vendoredSources = lib.fold lib.recursiveUpdate {} (lib.mapAttrsToList vendorSource remotePackagesBySource);

  cratesIORegistryURL = "https://github.com/rust-lang/crates.io-index";

  cratesIOSource = "registry+${cratesIORegistryURL}";

  mkCratesIOCrateTarballURL = { name, version }:
    "https://crates.io/api/v1/crates/${name}/${version}/download";

  mkCrateTarballURLFns = {
    "${cratesIORegistryURL}" = mkCratesIOCrateTarballURL;
  } // extraMkCrateTarballURLFns;

  vendorSource = source: packages:
    let
      parsedSource = parseSource source;
      vendorSourceFn = {
        registry = vendorRegistrySource;
        git = vendorGitSource;
      }.${parsedSource.type};
      vendoredSource = vendorSourceFn parsedSource.value packages;
      vendoredSourceName = "vendored+${source}";
      isCratesIO = source == cratesIOSource;
      guardedSource = if isCratesIO then "crates-io" else source;
    in {
      source.${vendoredSourceName}.directory = vendoredSource.directory;
      source.${guardedSource} = lib.optionalAttrs (!isCratesIO) vendoredSource.attrs // {
        replace-with = vendoredSourceName;
      };
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
        registry  = parseRegistrySource;
      }.${type} value;
    };

  parseRegistrySource = registrySource: {
    registryURL = registrySource;
  };

  parseGitSource = gitSource:
    let
      parts = builtins.match ''([^?]*)(\?(rev|tag|branch)=([^#&]*))?#(.*)'' gitSource;
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

  vendorRegistrySource = { registryURL }: packages: {
    attrs = {
        registry = registryURL;
    };
    directory = linkFarm "vendored-registry-source" (lib.forEach packages (package:
      let
        inherit (package) name version checksum;
      in {
        name = "${name}-${version}";
        path = fetchRegistryPackage {
          inherit registryURL name version checksum;
        };
      }
    ));
  };

  fetchRegistryPackage = { registryURL, name, version, checksum }:
    let
      tarball = fetchurl {
        name = "crate-${name}-${version}.tar.gz";
        url = mkCrateTarballURLFns.${registryURL} {
          inherit name version;
        };
        sha256 = checksum;
      };
    in
      runCommand "${name}-${version}" {} ''
        mkdir $out
        tar xf "${tarball}" -C $out --strip-components=1

        # Cargo is happy with partially empty metadata.
        printf '{"files": {}, "package": "${checksum}"}' > "$out/.cargo-checksum.json"
      '';

  vendorGitSource = { url, rev, ref ? null }: packages:
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
      #   allRefs = true; # HACK
      # });
    in {
      attrs = {
        git = url;
      } // lib.optionalAttrs (ref != null) {
        "${ref.type}" = ref.value;
      };
      directory = linkFarm "vendored-git-source" (lib.forEach packages (package:
        let
          inherit (package) name version;
        in {
          name = "${name}-${version}";
          path = extractGitPackage {
            inherit superTree rev name version;
          };
        }
      ));
    };

  extractGitPackage = { superTree, rev, name, version }:
    runCommand "${name}-${version}" {
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

      # HACK: Recreate some of .git for crates which try to detect whether they're a git dependency
      mkdir $out/.git
      echo "${rev}" > $out/.git/HEAD

      # Cargo is happy with empty metadata.
      printf '{"files": {}, "package": null}' > "$out/.cargo-checksum.json"
    '';

in {
  configFragment = vendoredSources;
}
