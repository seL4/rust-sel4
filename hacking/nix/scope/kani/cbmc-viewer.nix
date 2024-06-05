#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, buildPythonPackage
, fetchFromGitHub
, setuptools
, jinja2
, voluptuous
}:

buildPythonPackage rec {
  pname = "cbmc-viewer";
  version = "3.8";

  format = "pyproject";

  src = fetchFromGitHub {
    owner = "model-checking";
    repo = "cbmc-viewer";
    rev = "viewer-3.8";
    hash = "sha256-GIpinwjl/v6Dz5HyOsoPfM9fxG0poZ0HPsKLe9js9vM=";
  };

  propagatedBuildInputs = [
    setuptools
    jinja2
    voluptuous
  ];
}
