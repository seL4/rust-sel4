{ lib, rustPlatform, fetchFromGitHub, python3 }:

rustPlatform.buildRustPackage rec {
  name = "pyoxidizer";

  src = fetchFromGitHub {
    owner = "indygreg";
    repo = "PyOxidizer";
    rev = "36e50354c97275c7d9f47b3c38a8c216a08771f5";
    hash = "sha256-/nMEGxkL8HKXN/9AvSx9nda8EljarLwZhqifddKTvHI=";
  };

  cargoSha256 = "sha256-NoDa7WXUoKokdW6ac3r1OenbYdUJXbI7R5tr0lmSB7Y=";

  nativeBuildInputs = [
    python3
  ];

  doCheck = false;
}
