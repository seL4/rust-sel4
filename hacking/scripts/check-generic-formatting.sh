#!/bin/sh

set -eu

here=$(realpath $(dirname $0))
top_level_dir=$here/../..

ignore_patterns=" \
	-e ^docs/images/ \
	-e .ttf\$ \
	-e .patch\$ \
"

cd $top_level_dir

git ls-files | \
    grep -v $ignore_patterns | \
    (
        while read path; do
            [ -f "$path" ] && echo $path;
        done
    ) | \
    $(nix-build -A pkgs.build.python3 --no-out-link)/bin/python3 $here/check-generic-formatting-helper.py
