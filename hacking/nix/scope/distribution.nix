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
    sel4-loader
    sel4-loader-add-payload
    capdl-loader
    capdl-loader-add-spec
    capdl-loader-with-embedded-spec
  ];

  cratesForDistribution = crateUtils.getClosureOfCrates rootCratesForDistribution;

  orderCrates = theseCrates: (lib.toposort (a: b: lib.hasAttr a.name b.closure) theseCrates).result;

  orderedCrateNamesForDistribution = map (crate: crate.name) (orderCrates (lib.attrValues cratesForDistribution));

  showOrderedCrateNamesForDistribution = writeText "crates.txt" (lib.concatMapStrings (x: "${x}\n") orderedCrateNamesForDistribution);

in rec {
  inherit showOrderedCrateNamesForDistribution;
}
