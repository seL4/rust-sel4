{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-reserve-tls-on-stack";
  nix.local.dependencies = with localCrates; [
    sel4
  ];
  dependencies = {
    inherit (versions) cfg-if;
  };
}
