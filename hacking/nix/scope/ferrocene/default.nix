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

  mkToolchain = { hashesFilePath, arch, versionTag, versionName, componentNames ? defaultComponentNames }:
    let
      ferroceneSrc = mkFerroceneSrc {
        inherit hashesFilePath arch versionTag versionName;
      };

      byName = lib.fix (self: lib.listToAttrs (lib.forEach componentNames (componentName:
        let
          component = mkComponent {
            inherit componentName;
            inherit hashesFilePath arch versionTag versionName;
            inherit ferroceneSrc;
            inherit (self) rustc;
          };
        in
          lib.nameValuePair componentName component
      )));
    in
      fenix.combine (lib.attrValues byName);

  mkFiles = versionName: lib.mapAttrs (fname: sha256: requireFile {
    name = fname;
    url = "https://releases.ferrocene.dev/ferrocene/files/${versionName}/index.html";
    inherit sha256;
  });

  unpackComponent = { hashesFilePath, arch, versionTag, versionName, componentName }:
    let
      files = mkFiles versionName (parseHashesFile (builtins.readFile hashesFilePath));
      nameSuffix = "${
        lib.optionalString (!(lib.elem componentName [ "rust-src" "ferrocene-src" ])) "-${arch}"
      }-${versionTag}";
      fname = "${componentName}${nameSuffix}.tar.xz";
    in
      stdenv.mkDerivation {
        name = "${componentName}${nameSuffix}";
        src = files.${fname};
        sourceRoot = ".";
        installPhase = ''
          cp -r . $out
        '';
      }
  ;

  mkFerroceneSrc = { hashesFilePath, arch, versionTag, versionName }:
    (unpackComponent {
      inherit hashesFilePath arch versionTag versionName;
      componentName = "ferrocene-src";
    }).overrideAttrs (attrs: {
      dontConfigure = true;
      dontFixup = true;
      patchPhase = ''
        shopt -s extglob
        rm -r !(ferrocene)
      '';
    });

  rpath = "${zlib}/lib:$out/lib";

  mkComponent = { componentName, hashesFilePath, arch, versionTag, versionName, rustc, ferroceneSrc }:
    (unpackComponent {
      inherit componentName hashesFilePath arch versionTag versionName;
    }).overrideAttrs (attrs: {
      dontStrip = true;
      patchPhase = ''
        rm lib/rustlib/{components,install.log,manifest-*,rust-installer-version,uninstall.sh} || true

        ${lib.optionalString stdenv.isLinux ''
          if [ -d bin ]; then
            for file in $(find bin -type f); do
              if isELF "$file"; then
                patchelf \
                  --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                  --set-rpath ${rpath} \
                  "$file" || true
              fi
            done
          fi

          if [ -d lib ]; then
            for file in $(find lib -type f); do
              if isELF "$file"; then
                patchelf --set-rpath ${rpath} "$file" || true
              fi
            done
          fi

          if [ -d libexec ]; then
            for file in $(find libexec -type f); do
              if isELF "$file"; then
                patchelf \
                  --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                  --set-rpath ${rpath} \
                  "$file" || true
              fi
            done
          fi

          ${lib.optionalString (componentName == "rustc") ''
            for file in $(find lib/rustlib/*/bin -type f); do
              if isELF "$file"; then
                patchelf \
                  --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                  --set-rpath ${stdenv.cc.cc.lib}/lib:${rpath} \
                  "$file" || true
              fi
            done
          ''}

          ${lib.optionalString (componentName == "llvm-tools") ''
            for file in lib/rustlib/*/bin/*; do
              patchelf \
                --set-interpreter ${stdenv.cc.bintools.dynamicLinker} \
                --set-rpath lib/rustlib/*/lib \
                "$file" || true
            done
          ''}
        ''}

        ${lib.optionalString (componentName == "rustfmt") ''
          ${lib.optionalString stdenv.isLinux ''
            patchelf \
              --set-rpath ${rustc}/lib bin/rustfmt || true
          ''}
          ${lib.optionalString stdenv.isDarwin ''
            install_name_tool \
              -add_rpath ${rustc}/lib bin/rustfmt || true
          ''}
        ''}

        ${lib.optionalString (componentName == "rust-analyzer") ''
          ${lib.optionalString stdenv.isLinux ''
            patchelf \
              --set-rpath ${rustc}/lib bin/rust-analyzer || true
          ''}
          ${lib.optionalString stdenv.isDarwin ''
            install_name_tool \
              -add_rpath ${rustc}/lib bin/rust-analyzer || true
          ''}
        ''}

        ${lib.optionalString (componentName == "rust-src") ''
          substituteInPlace lib/rustlib/src/rust/library/Cargo.toml \
            --replace \
              ../ferrocene/library/libc \
              ${ferroceneSrc}/ferrocene/library/libc

          substituteInPlace lib/rustlib/src/rust/library/std/src/lib.rs \
            --replace \
              ../../../ferrocene/library/backtrace-rs/src/lib.rs \
              ${ferroceneSrc}/ferrocene/library/backtrace-rs/src/lib.rs
        ''}
      '';
    });


  defaultComponentNames = [
    "rustc"
    "rust-src"
    "rust-std"
    "cargo"
    "clippy"
    "llvm-tools"
    "rustfmt"
  ];

in

let
  # DO NOT CHECK IN THIS FILE
  hashesFilePath = ./SHA256SUMS;

  upstreamChannel = "1.88.0";
  upstreamDate = null;

  # versionTag = builtins.substring 0 9 versionRev;
  versionTag = "25.08.0";
  versionRev = "4a1d753a836d6f9a622289bb17734204cd70b151";
  versionName = "stable-${versionTag}";

  upstreamRustToolchain = assembleRustToolchain ({
    channel = upstreamChannel;
    sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
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
  };

  rustEnvironment = mkRustEnvironment rustToolchain;

in {
  inherit rustEnvironment;
  inherit upstreamRustEnvironment;
}
