{ stdenv, lib
, buildPackages, buildPlatform, hostPlatform
, fetchRepoProject
, cmake, ninja
, libxml2, dtc, cpio, protobuf
, python3Packages
, qemu
, git
, gccMultiStdenvGeneric
}:

with lib;

let
  thisStdenv = if hostPlatform.isRiscV64 then gccMultiStdenvGeneric else stdenv;
in

let
  initBuildArgs = [
    "-DSIMULATION=TRUE"
    "-DKernelSel4Arch=${hostPlatform.parsed.cpu.name}"
    "-DCROSS_COMPILER_PREFIX=${thisStdenv.cc.targetPrefix}"
    "-DMCS=FALSE"
    "-DSMP=FALSE"
  ] ++ lib.optionals hostPlatform.isRiscV64 [
    "-DPLATFORM=spike"
  ] ++ lib.optionals hostPlatform.isAarch64 [
    "-DPLATFORM=qemu-arm-virt"
    # "-DARM_HYP=TRUE"
  ];

in
thisStdenv.mkDerivation {
  name = "sel4test";

  src = fetchRepoProject {
    name = "sel4test";
    manifest = "https://github.com/seL4/sel4test-manifest.git";
    rev = "cfc1195ba8fd0de1a0e179aef1314b8f402ff74c";
    sha256 = "sha256-JspN1A/w5XIV+XCj5/oj7NABsKXVdr+UZOTJWvfJPUY=";
  };

  depsBuildBuild = lib.optionals (buildPlatform != hostPlatform) [
    buildPackages.stdenv.cc
    # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
    qemu
  ];

  nativeBuildInputs = [
    cmake ninja
    libxml2 dtc cpio protobuf
    git
  ] ++ (with python3Packages; [
    aenum plyplus pyelftools simpleeval
    sel4-deps
    buildPackages.python3Packages.protobuf
  ]);

  hardeningDisable = [ "all" ];

  postPatch = ''
    # patchShebangs can't handle env -S
    rm kernel/configs/*_verified.cmake
    rm tools/seL4/cmake-tool/helpers/cmakerepl

    patchShebangs --build .

    # HACK
    rm projects/musllibc/.git

    # HACK
    sed -i 's,--enable-static,--enable-static --disable-visibility,' \
      projects/musllibc/Makefile
  '';

  configurePhase = ''
    mkdir build
    cd build
    ../init-build.sh ${lib.concatStringsSep " " initBuildArgs}
  '';

  buildPhase = ''
    ninja
  '';

  installPhase = ''
    cd ..
    mv build $out
  '';

  dontFixup = true;

  enableParallelBuilding = true;
}
