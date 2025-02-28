#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, hostPlatform
, stdenv
, requireFile
, fetchFromGitHub
, zlib

, fenix
, assembleRustToolchain
, crateUtils
, elaborateRustEnvironment
, mkDefaultElaborateRustEnvironmentArgs
, mkMkCustomTargetPathForEnvironment
}:

let
  parseHashesFile = fileContents:
    let
      rawLines = lib.splitString "\n" fileContents;
      lines = assert lib.last rawLines == ""; lib.init rawLines;
      parseLine = line:
        let
          parts = builtins.match ''([0-9a-f]{64})  (.*)'' line;
          hash = lib.elemAt parts 0;
          fname = lib.elemAt parts 1;
        in
          lib.nameValuePair fname hash;
    in
      lib.listToAttrs (map parseLine lines);

  mkFiles = versionName: lib.mapAttrs (fname: sha256: requireFile {
    name = fname;
    url = "https://releases.ferrocene.dev/ferrocene/files/${versionName}/index.html";
    inherit sha256;
  });

  rpath = "${zlib}/lib:$out/lib";

  mkComponent = { componentName, nameSuffix, src, rustc, libcManifestDir }: stdenv.mkDerivation {
    name = "${componentName}${nameSuffix}";
    inherit src;
    sourceRoot = ".";
    installPhase = ''
      cp -r . $out

      rm $out/lib/rustlib/{components,install.log,manifest-*,rust-installer-version,uninstall.sh} || true

      ${lib.optionalString stdenv.isLinux ''
        if [ -d $out/bin ]; then
          for file in $(find $out/bin -type f); do
            if isELF "$file"; then
              patchelf \
                --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                --set-rpath ${rpath} \
                "$file" || true
            fi
          done
        fi

        if [ -d $out/lib ]; then
          for file in $(find $out/lib -type f); do
            if isELF "$file"; then
              patchelf --set-rpath ${rpath} "$file" || true
            fi
          done
        fi

        if [ -d $out/libexec ]; then
          for file in $(find $out/libexec -type f); do
            if isELF "$file"; then
              patchelf \
                --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                --set-rpath ${rpath} \
                "$file" || true
            fi
          done
        fi

        ${lib.optionalString (componentName == "rustc") ''
          for file in $(find $out/lib/rustlib/*/bin -type f); do
            if isELF "$file"; then
              patchelf \
                --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                --set-rpath ${stdenv.cc.cc.lib}/lib:${rpath} \
                "$file" || true
            fi
          done
        ''}

        ${lib.optionalString (componentName == "llvm-tools") ''
          for file in $out/lib/rustlib/*/bin/*; do
            patchelf \
              --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
              --set-rpath $out/lib/rustlib/*/lib \
              "$file" || true
          done
        ''}
      ''}

      ${lib.optionalString (componentName == "rustfmt") ''
        ${lib.optionalString stdenv.isLinux ''
          patchelf \
            --set-rpath ${rustc}/lib $out/bin/rustfmt || true
        ''}
        ${lib.optionalString stdenv.isDarwin ''
          install_name_tool \
            -add_rpath ${rustc}/lib $out/bin/rustfmt || true
        ''}
      ''}

      ${lib.optionalString (componentName == "rust-analyzer") ''
        ${lib.optionalString stdenv.isLinux ''
          patchelf \
            --set-rpath ${rustc}/lib $out/bin/rust-analyzer || true
        ''}
        ${lib.optionalString stdenv.isDarwin ''
          install_name_tool \
            -add_rpath ${rustc}/lib $out/bin/rust-analyzer || true
        ''}
      ''}

      ${lib.optionalString (componentName == "rust-src") ''
        substituteInPlace $out/lib/rustlib/src/rust/library/Cargo.toml \
          --replace ../ferrocene/library/libc ${libcManifestDir}
      ''}
    '';
    dontStrip = true;
  };

  defaultComponentNames = [
    "rustc"
    "rust-src"
    "rust-std"
    "cargo"
    "clippy"
    "llvm-tools"
    "rustfmt"
  ];

  mkToolchain = { hashesFilePath, arch, versionTag, versionName, libcManifestDir, componentNames ? defaultComponentNames }:
    let
      files = mkFiles versionName (parseHashesFile (builtins.readFile hashesFilePath));
      byName = lib.fix (self: lib.listToAttrs (lib.forEach componentNames (componentName:
        let
          nameSuffix = "${
            lib.optionalString (componentName != "rust-src") "-${arch}"
          }-${versionTag}";
          fname = "${componentName}${nameSuffix}.tar.xz";
          component = mkComponent {
            inherit componentName;
            inherit nameSuffix;
            src = files.${fname};
            inherit (self) rustc;
            inherit libcManifestDir;
          };
        in
          lib.nameValuePair componentName component
      )));
    in
      fenix.combine (lib.attrValues byName);
in

let
  # DO NOT CHECK IN THIS FILE
  hashesFilePath = ./SHA256SUMS;

  upstreamChannel = "1.83.0";
  upstreamDate = null;

  # versionTag = builtins.substring 0 9 versionRev;
  versionTag = "25.02.0";
  versionRev = "bd79e35e736899c87f46719491933af8224903bf";
  versionName = "stable-${versionTag}";

  repo = fetchFromGitHub {
    owner = "ferrocene";
    repo = "ferrocene";
    rev = versionRev;
    hash = "sha256-YR3shBZMpRpK3qkLO43Yz2Qcev/KQjeeErSDmAT2tBw=";
  };

  libcManifestDir = "${repo}/ferrocene/library/libc";

  upstreamRustToolchain = assembleRustToolchain ({
    channel = upstreamChannel;
    sha256 = "sha256-s1RPtyvDGJaX/BisLT+ifVfuhDT1nZkZ1NcK8sbwELM=";
  } // lib.optionalAttrs (upstreamDate != null) {
    date = upstreamDate;
  });

  mkRustEnvironment = rustToolchain: lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    channel = upstreamChannel;
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = upstreamRustEnvironment;
    };
  }));

  upstreamRustEnvironment = mkRustEnvironment upstreamRustToolchain;

  rustToolchain = mkToolchain {
    inherit hashesFilePath;
    arch = hostPlatform.config;
    inherit versionTag versionName;
    inherit libcManifestDir;
  };

  rustEnvironment = mkRustEnvironment rustToolchain;

in {
  inherit rustEnvironment;
  inherit upstreamRustEnvironment;
}
