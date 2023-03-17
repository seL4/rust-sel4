{ mkDerivation, aeson, array, base, base-compat, bytestring
, containers, filepath, lens, lib, MissingH, mtl, parsec, pretty
, regex-compat, split, text, unix, yaml
, sources
}:
mkDerivation {
  pname = "capDL-tool";
  version = "1.0.0.1";
  src = sources.capdlTool;
  isLibrary = false;
  isExecutable = true;
  executableHaskellDepends = [
    aeson array base base-compat bytestring containers filepath lens
    MissingH mtl parsec pretty regex-compat split text unix yaml
  ];
  homepage = "https://github.com/seL4/capdl";
  description = "A tool for processing seL4 capDL specifications";
  license = lib.licenses.bsd2;
  mainProgram = "parse-capDL";
}
