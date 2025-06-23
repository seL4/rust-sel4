{ mkDerivation, base, bytestring, deepseq, directory, hspec
, hspec-discover, HUnit, lib, QuickCheck, temporary
}:
mkDerivation {
  pname = "network";
  version = "3.1.4.0";
  sha256 = "b452a2afac95d9207357eb3820c719c7c7d27871ef4b6ed7bfcd03a036b9158e";
  revision = "1";
  editedCabalFile = "1vwxy5zj4bizgg2g0hk3dy52kjh5d7lzn33lphmvbbs36aqcslp1";
  libraryHaskellDepends = [ base bytestring deepseq directory ];
  testHaskellDepends = [
    base bytestring directory hspec HUnit QuickCheck temporary
  ];
  testToolDepends = [ hspec-discover ];
  homepage = "https://github.com/haskell/network";
  description = "Low-level networking interface";
  license = lib.licenses.bsd3;
}
