{ lib }:

let
  mkType = type: value: {
    inherit type value;
  };

in rec {
  mkString = mkType "STRING";
  mkBool = mkType "BOOL";
  on = mkBool "ON";
  off = mkBool "OFF";

  fromBool = x: if x then on else off;
}
