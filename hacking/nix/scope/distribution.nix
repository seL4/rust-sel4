{ lib
, writeText
, crateUtils
, crates
}:

let
  rootCratesForDistribution = with crates; [
    sel4
    sel4cp
    sel4-root-task
    sel4-kernel-loader
    sel4-kernel-loader-add-payload
    sel4-capdl-initializer
    sel4-capdl-initializer-add-spec
    sel4-capdl-initializer-with-embedded-spec
  ];

  cratesForDistribution = crateUtils.getClosureOfCrates rootCratesForDistribution;

  orderCrates = theseCrates: (lib.toposort (a: b: lib.hasAttr a.name b.closure) theseCrates).result;

  orderedCrateNamesForDistribution = map (crate: crate.name) (orderCrates (lib.attrValues cratesForDistribution));

  showOrderedCrateNamesForDistribution = writeText "crates.txt" (lib.concatMapStrings (x: "${x}\n") orderedCrateNamesForDistribution);

in rec {
  inherit showOrderedCrateNamesForDistribution;
}
