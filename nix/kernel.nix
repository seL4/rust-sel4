{ lib, stdenv, writeText, buildPackages
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemu
}:

{ config }:

let
  # src =../../../../../../local/seL4;

  src = builtins.fetchGit {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    ref = "rust";
    rev = "4a980702a820c6a2aed461e0d02cdbfb5a23749d";
  };

  settings = writeText "settings.cmake" ''
    ${lib.concatStrings (lib.mapAttrsToList (k: v: ''
      set(${k} ${v.value} CACHE ${v.type} "")
    '') config)}
  '';

in
stdenv.mkDerivation {
  name = "seL4";

  inherit src;

  nativeBuildInputs = [
    cmake ninja
    dtc libxml2
  ];

  depsBuildBuild = [
    qemu

    # NOTE: use buildPackages to work around splicing issue
    buildPackages.python3Packages.sel4-deps
  ];

  hardeningDisable = [ "all" ];

  postPatch = ''
    # patchShebangs can't handle env -S
    rm configs/*_verified.cmake

    patchShebangs --build .
  '';

  configurePhase = ''
    build=$(pwd)/build
    install_prefix=$(pwd)/install

    cmake \
      -DCROSS_COMPILER_PREFIX=${stdenv.cc.targetPrefix} \
      -DCMAKE_TOOLCHAIN_FILE=gcc.cmake \
      -DCMAKE_INSTALL_PREFIX=$install_prefix \
      -C ${settings} \
      -G Ninja \
      -B $build
  '';

  buildPhase = ''
    ninja -C $build all
  '';  

  installPhase = ''
    ninja -C $build install

    mkdir $out

    mv $install_prefix/libsel4/include $out

    install -D -t $out/boot \
      $build/kernel.dtb \
      $install_prefix/bin/kernel.elf

    install -D -t $out/sel4-aux \
      $build/gen_config/kernel/gen_config.json \
      $build/gen_headers/plat/machine/platform_gen.yaml
  '';

  dontFixup = true;
}
