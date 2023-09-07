{ lib
, stdenv
, fetchFromGitHub
, python3
}:

stdenv.mkDerivation rec {
  pname = "opensbi";
  version = "1.3.1";

  src = fetchFromGitHub {
    owner = "riscv-software-src";
    repo = "opensbi";
    rev = "v${version}";
    hash = "sha256-JNkPvmKYd5xbGB2lsZKWrpI6rBIckWbkLYu98bw7+QY=";
  };

  postPatch = ''
    patchShebangs ./scripts
  '';

  nativeBuildInputs = [ python3 ];

  makeFlags = [
    "PLATFORM=generic"
  ];

  installFlags = [
    "I=$(out)"
  ];

  dontFixup = true;
}
