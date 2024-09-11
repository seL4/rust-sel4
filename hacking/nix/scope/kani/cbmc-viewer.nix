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
  version = "3.9";

  format = "pyproject";

  src = fetchFromGitHub {
    owner = "model-checking";
    repo = "cbmc-viewer";
    rev = "viewer-3.9";
    hash = "sha256-BfXusrOXGBvquM841K4gb5HQVSryiZS8+ihgj7DVxbI=";
  };

  propagatedBuildInputs = [
    setuptools
    jinja2
    voluptuous
  ];
}
