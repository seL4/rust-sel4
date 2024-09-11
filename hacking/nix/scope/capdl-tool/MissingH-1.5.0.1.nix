{ mkDerivation, array, base, containers, directory, filepath
, hslogger, HUnit, lib, mtl, network, network-bsd, old-locale
, old-time, parsec, process, regex-compat, time, unix
}:
mkDerivation {
  pname = "MissingH";
  version = "1.5.0.1";
  sha256 = "cb2fa4a62a609ec6bcfa2eab6ae6d34c6f5bfba523fed8dc0c055b3176732231";
  revision = "2";
  editedCabalFile = "11d922r06p00gcgzhb29hhjkq8ajy1xbqdiwdpbmhp2ar7fw7g9l";
  libraryHaskellDepends = [
    array base containers directory filepath hslogger mtl network
    network-bsd old-locale old-time parsec process regex-compat time
    unix
  ];
  testHaskellDepends = [
    base containers directory filepath HUnit old-time parsec
    regex-compat time unix
  ];
  description = "Large utility library";
  license = lib.licenses.bsd3;
}
