{ mk, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-externally-shared";
  package.license = "MIT";
  package.authors = [ meAsAuthor ];
  features = {
    unstable = [];
    alloc = [];
  };
}
