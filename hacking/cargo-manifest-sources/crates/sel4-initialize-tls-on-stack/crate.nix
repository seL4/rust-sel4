{ mk, localCrates, versions }:

mk {
  package.name = "sel4-initialize-tls-on-stack";
  dependencies = {
    inherit (versions) cfg-if;
  };
  nix.local.dependencies = with localCrates; [
    sel4
  ];
  nix.meta.requirements = [ "sel4" ];
}
