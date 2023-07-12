self: super: with self;

let
  scopeName = "this";
in

assert !(super ? scopeName);

{

  "${scopeName}" =
    let
      otherSplices = generateSplicesForMkScope scopeName;
    in
      lib.makeScopeWithSplicing
        splicePackages
        newScope
        otherSplices
        (_: {})
        (_: {})
        (self: callPackage ../scope {} self // {
          __dontMashWhenSplicingChildren = true;
          inherit otherSplices; # for child spliced scopes
        })
      ;

  stdenv =
    if super.stdenv.hostPlatform.isNone && !(super.stdenv.hostPlatform.this.noneWithLibc or false)
    then
      # Use toolchain without newlib. This is equivalent to crossLibcStdenv.
      super.overrideCC super.stdenv super.crossLibcStdenv.cc
    else
      super.stdenv;

  # Add Python packages needed by the seL4 ecosystem
  pythonPackagesExtensions = super.pythonPackagesExtensions ++ [
    (callPackage ./python-overrides.nix {})
  ];

  gccMultiStdenvGeneric = overrideCC stdenv (buildPackages.wrapCC (stdenv.cc.cc.override {
    enableMultilib = true;
  }));

  qemu = super.qemu.overrideDerivation (attrs: {
    patches = attrs.patches ++ [
      (fetchurl {
        url = "https://github.com/coliasgroup/qemu/commit/cd3b78de4b5a8d7c79ae99dab2b5e0ab1ba0ffac.patch";
        sha256 = "sha256-bDmMyelaMCJWhr88XIKEBNMZP3VcBD3mOXhOWal3IBw=";
      })
    ];
  });

}
