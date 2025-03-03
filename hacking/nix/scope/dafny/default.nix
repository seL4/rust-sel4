#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, buildDotnetModule
, fetchFromGitHub
, writeScript
, dotnet-sdk_6
, jdk11
, z3
, sources
}:

let
  f = { useLocal ? false }: buildDotnetModule rec {
    pname = "Dafny";
    version = "4.6.0";

    src = sources.fetchGit {
      url = "https://github.com/coliasgroup/dafny.git";
      rev = "02d0a578fdf594a38c7c72d7ad56e1a62e464f44";
      local = sources.localRoot + "/dafny";
      # inherit useLocal;
    };

    postPatch = ''
      cp ${
        writeScript "fake-gradlew-for-dafny" ''
          mkdir -p build/libs/
          javac $(find -name "*.java" | grep "^./src/main") -d classes
          jar cf build/libs/DafnyRuntime-${version}.jar -C classes dafny
        ''} Source/DafnyRuntime/DafnyRuntimeJava/gradlew

      # Needed to fix
      # "error NETSDK1129: The 'Publish' target is not supported without
      # specifying a target framework. The current project targets multiple
      # frameworks, you must specify the framework for the published
      # application."
      substituteInPlace Source/DafnyRuntime/DafnyRuntime.csproj \
        --replace TargetFrameworks TargetFramework \
        --replace "netstandard2.0;net452" net6.0
    '';

    dotnet-sdk = dotnet-sdk_6;

    nativeBuildInputs = [
      jdk11
    ];

    nugetDeps = ./deps.nix;

    # Build just these projects. Building Source/Dafny.sln includes a bunch of
    # unnecessary components like tests.
    projectFile = [
      "Source/Dafny/Dafny.csproj"
      "Source/DafnyRuntime/DafnyRuntime.csproj"
      # "Source/DafnyLanguageServer/DafnyLanguageServer.csproj"
    ];

    executables = [ "Dafny" ];

    # Help Dafny find z3
    makeWrapperArgs = [ "--prefix PATH : ${lib.makeBinPath [ z3 ]}" ];

    postFixup = ''
      ln -s "$out/bin/Dafny" "$out/bin/dafny" || true
    '';

    passthru = {
      local = f { useLocal = true; };
      remote = f { useLocal = false; };
    };
  };

in f {
  # useLocal = true;
}
