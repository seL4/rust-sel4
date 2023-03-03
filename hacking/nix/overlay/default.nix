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
    # Use toolchain without newlib. This is equivalent to crossLibcStdenv.
    then super.overrideCC super.stdenv super.crossLibcStdenv.cc
    else super.stdenv;

  # Add Python packages needed by the seL4 ecosystem
  pythonPackagesExtensions = super.pythonPackagesExtensions ++ [
    (callPackage ./python-overrides.nix {})
  ];

  gccMultiStdenvGeneric =
    let
      # stdenv = super.gcc8Stdenv;
      # stdenv = super.gcc10Stdenv;
    in
      overrideCC stdenv (buildPackages.wrapCC (stdenv.cc.cc.override {
        enableMultilib = true;
      }));

}
