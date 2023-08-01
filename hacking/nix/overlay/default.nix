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

  # HACK: something wrong with multilib for riscv beyond gcc11 in nixpkgs
  gccMultiStdenvGeneric = overrideCC stdenv (buildPackages.wrapCC (gcc11Stdenv.cc.cc.override {
    enableMultilib = true;
  }));

}
