{ lib, emptyDirectory, linkFarm, writeText }:

{
  passthru = {
    spec = writeText "x.cdl" ''
      arch aarch64

      objects {
        foo = notification
      }
      caps {}
    '';
    fill = emptyDirectory;
  };
}
